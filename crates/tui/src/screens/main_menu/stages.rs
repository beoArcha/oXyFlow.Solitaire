//! The stage changes of the main menu: what a game list, a seed screen and the Back key do to the
//! stage, and the start effect of a seed chosen on the seed screen.

use super::seed_screen::{Outcome, SeedScreen};
use super::{GameSelect, MainMenu, Stage};
use crate::screens::{Effect, ViewCtx};

impl MainMenu {
    pub(super) fn leave_variants_on_start(&mut self, effect: Effect) -> Effect {
        if matches!(effect, Effect::Start { .. }) {
            self.stage = Stage::Items;
        }
        effect
    }

    /// The game list of "New game from seed" starts nothing: a chosen game opens its seed screen.
    pub(super) fn after_game(&mut self, effect: Effect) -> Effect {
        match effect {
            Effect::Start { variant, .. } => {
                self.stage = Stage::Seed(SeedScreen::new(variant, self.played.of_variant(variant)));
                Effect::None
            }
            other => other,
        }
    }

    /// Turns the seed screen's outcome into the menu's stage and, for a seed, the start effect.
    pub(super) fn after_seed(&mut self, outcome: Outcome, ctx: &ViewCtx<'_>) -> Effect {
        match outcome {
            Outcome::Stay => Effect::None,
            Outcome::Leave => {
                self.stage = Stage::Game(GameSelect::before_seed());
                Effect::None
            }
            Outcome::Start { variant, seed } => {
                self.stage = Stage::Items;
                Effect::Start {
                    variant,
                    seed: Some(seed),
                    undo_allowed: ctx.settings.undo_allowed,
                }
            }
            Outcome::Replay {
                variant,
                seed,
                options,
                undo_allowed,
            } => {
                self.stage = Stage::Items;
                Effect::Replay {
                    variant,
                    seed,
                    options,
                    undo_allowed,
                }
            }
        }
    }

    /// Esc or Back: a seed screen goes back one page (the game list from its options), every other
    /// stage goes back to the items. The games of a command-line seed have no earlier page.
    pub(super) fn back(&mut self, ctx: &ViewCtx<'_>) -> Effect {
        match &mut self.stage {
            Stage::Seed(screen) => {
                let outcome = screen.back();
                self.after_seed(outcome, ctx)
            }
            _ => {
                self.stage = Stage::Items;
                Effect::None
            }
        }
    }
}
