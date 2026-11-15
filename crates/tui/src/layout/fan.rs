//! Fanning of a tableau column so that it always fits the room below the top row.
//!
//! Compression is progressive and monotone: face-down steps shrink first, then face-up steps, then
//! the face-down cards collapse into one row with a counter. Only when even that cannot fit, the
//! oldest face-up cards stack up under each other; the newest cards keep their ranks visible.

use oxysol_core::TableCard;

use super::size::CardSize;

/// Vertical offsets of the cards of one column, from the first card downwards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fan {
    pub ys: Vec<u16>,
    /// Leading face-down cards drawn as one row.
    pub collapsed: usize,
}

#[derive(Debug, Clone, Copy)]
struct Plan {
    down: u16,
    up: u16,
    collapse: bool,
}

fn leading_face_down(cards: &[TableCard]) -> usize {
    cards.iter().take_while(|card| !card.face_up).count()
}

fn offsets(cards: &[TableCard], plan: Plan, prefix: usize) -> Vec<u16> {
    let mut y = 0;
    let mut ys = Vec::with_capacity(cards.len());
    for (index, card) in cards.iter().enumerate() {
        if plan.collapse && index < prefix {
            ys.push(0);
            y = 1;
            continue;
        }
        ys.push(y);
        y += if card.face_up { plan.up } else { plan.down };
    }
    ys
}

fn fits(ys: &[u16], height: u16, room: u16) -> bool {
    ys.last().is_none_or(|last| last + height <= room)
}

fn plans(size: CardSize, collapsible: bool) -> Vec<Plan> {
    let (up, down) = (size.up_step(), size.down_step());
    let shrink_down = (1..=down).rev().map(|down| Plan {
        down,
        up,
        collapse: false,
    });
    let shrink_up = (1..up).rev().map(|up| Plan {
        down: 1,
        up,
        collapse: false,
    });
    let collapse = (1..=up).rev().map(|up| Plan {
        down: 1,
        up,
        collapse: true,
    });
    let mut all: Vec<Plan> = shrink_down.chain(shrink_up).collect();
    if collapsible {
        all.extend(collapse);
    }
    all
}

fn squeezed(count: usize, prefix: usize, collapse: bool, height: u16, room: u16) -> Vec<u16> {
    let floor = u16::from(collapse);
    let last = room.saturating_sub(height);
    (0..count)
        .map(|index| {
            if collapse && index < prefix {
                return 0;
            }
            let behind = u16::try_from(count - 1 - index).unwrap_or(u16::MAX);
            last.saturating_sub(behind).max(floor)
        })
        .collect()
}

/// Offsets for `cards` (bottom to top) in a column of `room` rows whose cards are `size` tall.
pub fn fan(cards: &[TableCard], size: CardSize, room: u16) -> Fan {
    let prefix = leading_face_down(cards);
    let collapsible = prefix >= 2 && prefix < cards.len();
    for plan in plans(size, collapsible) {
        let ys = offsets(cards, plan, prefix);
        if fits(&ys, size.height, room) {
            let collapsed = if plan.collapse { prefix } else { 0 };
            return Fan { ys, collapsed };
        }
    }
    let collapsed = if collapsible { prefix } else { 0 };
    Fan {
        ys: squeezed(cards.len(), prefix, collapsible, size.height, room),
        collapsed,
    }
}
