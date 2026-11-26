//! Acceptance tests for the no-moves-left dialog across TriPeaks, Golf, a blocked tableau game,
//! button actions, Esc dismissal, and all offered locales at 100×30.

use std::time::Instant;

use oxysol_core::solver::{SolveBudget, Verdict, solve};
use oxysol_core::{DrawCount, Move, Session, Variant, YukonRules};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::style::Modifier;

use super::no_moves::text;
use crate::app::{FontFacts, Modal, Screen};
use crate::audio::AudioStatus;
use crate::caps::{ColorDepth, TerminalCaps};
use crate::glyphs::{Glyphs, is_wgl4};
use crate::i18n::{self, Locale, t};
use crate::layout::{self, InterfaceSize};
use crate::screens::board::BoardScreen;
use crate::screens::{Effect, ViewCtx};
use crate::settings::Settings;
use crate::storage::memory::{HistoryRow, MemoryStorage};
use crate::storage::{GameId, SavedGame};
use crate::test_support::{Harness, TRUE_COLOR};
use crate::theme::{Theme, ThemeId};
use crate::widgets::overlay;

const AREA: Rect = Rect::new(0, 0, 100, 30);

fn find_blocked_stock_game(variant: Variant) -> (u64, Vec<Move>) {
    (0..200_u64)
        .find_map(|seed| {
            let mut session = Session::new(variant, seed);
            let mut moves = Vec::new();
            while session.game().legal_moves().contains(&Move::Stock) {
                session.apply(Move::Stock).ok()?;
                moves.push(Move::Stock);
            }
            (!session.game().is_won() && session.game().legal_moves().is_empty())
                .then_some((seed, moves))
        })
        .expect("a deal runs out of moves after exhausting the stock")
}

fn find_blocked_tableau_game() -> (Variant, u64, Vec<Move>) {
    let variant = Variant::Yukon(YukonRules::Russian);
    (0..200_u64)
        .find_map(|seed| {
            let mut session = Session::new(variant, seed);
            let mut moves = Vec::new();
            for _ in 0..40 {
                let Some(&mv) = session.game().legal_moves().first() else {
                    break;
                };
                session.apply(mv).ok()?;
                moves.push(mv);
            }
            (!moves.is_empty()
                && !session.game().is_won()
                && session.game().legal_moves().is_empty())
            .then_some((variant, seed, moves))
        })
        .expect("some Russian Solitaire deal blocks")
}

fn resumed(variant: Variant, seed: u64, moves: &[Move], undo_allowed: bool) -> Harness {
    let mut session = Session::new(variant, seed);
    for mv in moves {
        assert_eq!(session.apply(*mv), Ok(()));
    }
    let mut row = HistoryRow::open(1, variant, seed);
    row.undo_allowed = undo_allowed;
    let storage = MemoryStorage {
        game: Some(SavedGame {
            id: GameId(1),
            record: session.record(),
            elapsed_ms: 0,
            undo_allowed,
        }),
        history: vec![row],
        ..MemoryStorage::default()
    };
    let mut h = Harness::with(storage, TRUE_COLOR, 100, 30);
    h.app.apply(Effect::Resume);
    h
}

fn play_last_move(h: &mut Harness, mv: Move) {
    h.lines();
    match mv {
        Move::Stock => h.type_char('s'),
        Move::Transfer { from, to, count } => {
            let session = h.app.board().session().expect("running session");
            let piles = session.game().piles();
            let body = layout::chrome_panels_for(AREA, true, true, false, InterfaceSize::M).body;
            let geometry = layout::compute(session.variant(), &piles, body);
            let from_pile = geometry.pile(from).expect("source pile");
            let from_rect = from_pile.cards[from_pile.cards.len() - usize::from(count)];
            let to_pile = geometry.pile(to).expect("target pile");
            let to_rect = to_pile.cards.last().copied().unwrap_or(to_pile.slot);
            h.click(from_rect.x + 1, from_rect.y);
            h.click(to_rect.x + 1, to_rect.y);
        }
        Move::Remove { .. } => unreachable!(),
    }
}

fn assert_locale_fits(locale: Locale, unicode: bool, can_undo: bool) {
    i18n::set_locale(locale);
    let caps = TerminalCaps {
        color: ColorDepth::TrueColor,
        unicode,
        remote: false,
    };
    let settings = Settings {
        locale,
        ..Settings::for_caps(&caps)
    };
    let theme = Theme::new(ThemeId::Dark, ColorDepth::TrueColor);
    let glyphs = Glyphs::new(settings.drawn_glyph_mode(false));
    let board = BoardScreen::new();
    let ctx = ViewCtx {
        theme: &theme,
        glyphs: &glyphs,
        settings: &settings,
        caps: &caps,
        audio_status: AudioStatus::NotBuilt,
        sound_styles: &[],
        saved: None,
        pointer: None,
        now: Instant::now(),
        font: FontFacts::default(),
    };
    let modal = Modal::no_moves(can_undo);
    let mut terminal = Terminal::new(TestBackend::new(AREA.width, AREA.height)).expect("term");
    terminal
        .draw(|frame| {
            overlay::draw(frame, frame.area(), &modal, &board, &ctx);
        })
        .expect("draw");
    let buf = terminal.backend().buffer();
    let rows: Vec<String> = (0..AREA.height)
        .map(|r| (0..AREA.width).map(|c| buf[(c, r)].symbol()).collect())
        .collect();
    let full = rows.join("\n");
    let geometry = text(can_undo, 0).geometry_for(AREA, InterfaceSize::M);
    let body_text: String = (geometry.body.y..geometry.body.bottom())
        .map(|r| {
            (geometry.body.x..geometry.body.right())
                .map(|c| buf[(c, r)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ");
    let normalized = body_text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        full.contains(&*t("board.no_moves.title")),
        "{}",
        locale.id()
    );
    assert!(
        normalized.contains(&*t("board.no_moves.body")),
        "{}",
        locale.id()
    );
    let undo = t("board.moves.label.undo");
    assert_eq!(full.contains(&*undo), can_undo, "{}", locale.id());
    for key in [
        "help.overlay.restart",
        "help.overlay.new_game",
        "help.overlay.menu",
    ] {
        assert!(full.contains(&*t(key)), "{} {key}", locale.id());
    }
    let first = if can_undo {
        undo
    } else {
        t("help.overlay.restart")
    };
    let marker = if unicode {
        format!("► {first} ◄")
    } else {
        format!("> {first} <")
    };
    let (col, row) = rows
        .iter()
        .enumerate()
        .find_map(|(r, line)| {
            let byte = line.find(&marker)?;
            Some((
                u16::try_from(line[..byte].chars().count()).ok()?,
                u16::try_from(r).ok()?,
            ))
        })
        .expect("focused button marker");
    assert!(
        buf[(col, row)]
            .modifier
            .contains(Modifier::BOLD | Modifier::REVERSED)
    );
    for ch in full.chars().filter(|ch| *ch != '\n') {
        if unicode {
            assert!(ch.is_ascii() || is_wgl4(ch));
        } else if locale == Locale::EnUs {
            assert!(ch.is_ascii(), "{locale:?}: {ch:?}");
        }
    }
}

#[test]
fn tripeaks_and_golf_open_no_moves_only_after_the_final_move() {
    for variant in [Variant::TriPeaks, Variant::Golf] {
        let (seed, moves) = find_blocked_stock_game(variant);
        let (last, earlier) = moves.split_last().expect("at least one move");
        let mut h = resumed(variant, seed, earlier, true);
        assert!(h.app.modal().is_none(), "{variant:?} open before last move");
        play_last_move(&mut h, *last);
        assert_eq!(h.app.modal(), Some(&Modal::no_moves(true)), "{variant:?}");
        let screen = h.text();
        assert!(screen.contains("No moves left"), "{screen}");
        assert!(
            screen.contains("No move is possible. This game can no longer be won."),
            "{screen}"
        );
    }
}

#[test]
fn tableau_game_opens_no_moves_and_won_game_opens_win_instead() {
    let (variant, seed, moves) = find_blocked_tableau_game();
    let (last, earlier) = moves.split_last().expect("at least one move");
    let mut h = resumed(variant, seed, earlier, true);
    assert!(h.app.modal().is_none());
    play_last_move(&mut h, *last);
    assert_eq!(h.app.modal(), Some(&Modal::no_moves(true)));

    let klondike = Variant::Klondike(DrawCount::One);
    let Verdict::Winnable { moves: win_moves } =
        solve(klondike, 71, SolveBudget::default_for(klondike))
    else {
        unreachable!("klondike seed 71 is winnable");
    };
    let (win_last, win_earlier) = win_moves.split_last().expect("win moves");
    let mut h = resumed(klondike, 71, win_earlier, true);
    play_last_move(&mut h, *win_last);
    assert_eq!(h.app.modal(), Some(&Modal::win()));
}

#[test]
fn undo_button_appears_only_when_it_can_act_and_closes_dialog() {
    let (seed, moves) = find_blocked_stock_game(Variant::TriPeaks);
    let (last, earlier) = moves.split_last().expect("at least one move");
    let mut h = resumed(Variant::TriPeaks, seed, earlier, true);
    play_last_move(&mut h, *last);
    assert!(h.text().contains("► Undo ◄"));
    let before = h.app.board().session().expect("session").moves_made();
    h.press(KeyCode::Enter);
    assert!(h.app.modal().is_none());
    let session = h.app.board().session().expect("session");
    assert_eq!(session.moves_made(), before - 1);
    assert!(!session.game().legal_moves().is_empty());

    let mut h_no_undo = resumed(Variant::TriPeaks, seed, earlier, false);
    play_last_move(&mut h_no_undo, *last);
    assert_eq!(h_no_undo.app.modal(), Some(&Modal::no_moves(false)));
    let screen = h_no_undo.text();
    assert!(!screen.contains("Undo"), "{screen}");
    assert!(screen.contains("► Restart ◄"), "{screen}");
}

#[test]
fn restart_new_game_and_menu_buttons_work_by_key_and_mouse() {
    let (seed, moves) = find_blocked_stock_game(Variant::Golf);
    let (last, earlier) = moves.split_last().expect("at least one move");

    let mut h_restart = resumed(Variant::Golf, seed, earlier, false);
    play_last_move(&mut h_restart, *last);
    let (col, row) = h_restart.find("Restart").expect("Restart button");
    h_restart.click(col, row);
    assert!(h_restart.app.modal().is_none());
    assert_eq!(h_restart.app.screen(), Screen::Board);
    assert_eq!(h_restart.app.board().restarts(), 1);
    assert_eq!(h_restart.storage.borrow().history[0].counters.restarts, 1);
    let session = h_restart.app.board().session().expect("session");
    assert_eq!((session.seed(), session.moves_made()), (seed, 0));

    let mut h_new_game = resumed(Variant::Golf, seed, earlier, false);
    play_last_move(&mut h_new_game, *last);
    h_new_game.press(KeyCode::Tab);
    h_new_game.press(KeyCode::Enter);
    assert!(h_new_game.app.modal().is_none());
    assert_eq!(h_new_game.app.screen(), Screen::GameSelect);

    let mut h_menu = resumed(Variant::Golf, seed, earlier, true);
    play_last_move(&mut h_menu, *last);
    h_menu.press(KeyCode::BackTab);
    h_menu.press(KeyCode::Enter);
    assert!(h_menu.app.modal().is_none());
    assert_eq!(h_menu.app.screen(), Screen::MainMenu);
}

#[test]
fn esc_keeps_board_and_stuck_bar_and_never_reopens_for_same_position() {
    let (seed, moves) = find_blocked_stock_game(Variant::TriPeaks);
    let (last, earlier) = moves.split_last().expect("at least one move");
    let mut h = resumed(Variant::TriPeaks, seed, earlier, true);
    play_last_move(&mut h, *last);
    assert!(matches!(h.app.modal(), Some(Modal::NoMoves { .. })));

    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    assert_eq!(h.app.screen(), Screen::Board);
    let screen = h.text();
    assert!(screen.contains("Now: No legal moves"), "{screen}");
    assert!(screen.contains("u: undo"), "{screen}");
    assert!(screen.contains("n: new game"), "{screen}");

    for key in [KeyCode::Left, KeyCode::Right, KeyCode::Enter] {
        h.press(key);
        assert!(h.app.modal().is_none());
    }
    for ch in ['s', 'a', 'h', 'f', 'r'] {
        h.type_char(ch);
        assert!(h.app.modal().is_none());
    }
    h.type_char('?');
    assert!(matches!(h.app.modal(), Some(Modal::Help(_))));
    h.press(KeyCode::Esc);
    assert!(h.app.modal().is_none());
    assert!(
        resumed(Variant::TriPeaks, seed, &moves, true)
            .app
            .modal()
            .is_none()
    );
}

#[test]
fn every_offered_locale_fits_at_100_by_30_with_visible_focus() {
    for locale in Locale::offered() {
        for unicode in [true, false] {
            assert_locale_fits(locale, unicode, true);
            assert_locale_fits(locale, unicode, false);
        }
    }
    i18n::set_locale(Locale::EnUs);
}
