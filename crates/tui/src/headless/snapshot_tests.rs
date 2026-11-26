use std::time::Instant;

use oxysol_core::{DrawCount, Variant};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;

use crate::app::{App, Deps, DisplayDeps, Modal, SearchProgress, Zone};
use crate::audio::{AudioStatus, NullAudio};
use crate::clipboard::NullClipboard;
use crate::glyphs::Glyphs;
use crate::screens::Effect;
use crate::screens::ViewCtx;
use crate::screens::board::BoardScreen;
use crate::settings::{DockSide, Settings};
use crate::solver::NullSolver;
use crate::storage::memory::MemoryStorage;
use crate::test_support::{TEST_VERSION, TRUE_COLOR, sound_styles};
use crate::theme::{Theme, UserTheme};
use crate::widgets::overlay::{
    self, CommandLine, GraphicsCheck, GuideWindow, HowToPlay, MovesWindow, QuickOptions, Rules,
};

#[test]
fn overlay_rect_matches_drawn_border_for_every_modal() {
    let area = Rect::new(0, 0, 140, 40);
    let now = Instant::now();
    let board = BoardScreen::new();
    let mut settings = Settings::for_caps(&TRUE_COLOR);
    let theme = Theme::for_settings(settings.theme, None, settings.color_depth(&TRUE_COLOR));
    let glyphs = Glyphs::styled(
        settings.drawn_glyph_mode(false),
        &settings.front_style,
        &settings.back_style,
    );
    let styles = sound_styles();

    let modals = [
        Modal::help(),
        Modal::HowToPlay(HowToPlay::new(Rules::Klondike)),
        Modal::CommandLine(CommandLine::new()),
        Modal::Guide(GuideWindow::new(20)),
        Modal::Moves(MovesWindow::new(Vec::new())),
        Modal::Quick(QuickOptions::new(settings.aids())),
        Modal::pause(true),
        Modal::confirm("board.confirm.quit", Effect::Quit),
        Modal::GraphicsCheck(GraphicsCheck::new(&settings, false, Effect::None)),
        Modal::alert("common.error.storage"),
        Modal::Search(SearchProgress {
            variant: Variant::Klondike(DrawCount::One),
            tried: 3,
            cap: 40,
            capped: false,
            can_play_unproven: false,
            focus: 0,
        }),
        Modal::win(),
    ];

    for modal in &modals {
        let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
        let ctx = ViewCtx {
            theme: &theme,
            glyphs: &glyphs,
            settings: &settings,
            caps: &TRUE_COLOR,
            audio_status: AudioStatus::NotBuilt,
            sound_styles: &styles,
            saved: None,
            pointer: None,
            now,
            font: Default::default(),
        };
        terminal
            .draw(|frame| {
                overlay::draw(frame, area, modal, &board, &ctx);
            })
            .unwrap();
        let rect = overlay::rect_for(
            area,
            modal,
            &board,
            now,
            DockSide::Centre,
            settings.interface_size,
        )
        .expect("centered modal should have a rect");
        let buf = terminal.backend().buffer();
        assert_eq!(
            buf[(rect.x, rect.y)].symbol(),
            "╔",
            "top-left for {modal:?}"
        );
        assert_eq!(
            buf[(rect.right() - 1, rect.y)].symbol(),
            "╗",
            "top-right for {modal:?}"
        );
        assert_eq!(
            buf[(rect.x, rect.bottom() - 1)].symbol(),
            "╚",
            "bottom-left for {modal:?}"
        );
        assert_eq!(
            buf[(rect.right() - 1, rect.bottom() - 1)].symbol(),
            "╝",
            "bottom-right for {modal:?}"
        );
    }

    settings.help_dock = DockSide::Right;
    let mut app = App::new(Deps {
        storage: Box::new(MemoryStorage::new()),
        audio: Box::new(NullAudio::new(AudioStatus::NotBuilt)),
        clipboard: Box::new(NullClipboard),
        caps: TRUE_COLOR,
        seeds: Box::new(|| 1),
        start_seed: None,
        sound_styles: styles,
        user_theme: UserTheme::Missing,
        solver: Box::new(NullSolver::default()),
        display: DisplayDeps::none(),
        version: TEST_VERSION,
        zone: Zone::Utc,
    });
    app.apply(Effect::Apply(settings));
    app.apply(Effect::Open(Modal::help()));
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let snap = app.snapshot();
    assert!(snap.dock_mode.is_some_and(|d| d.docked));
    let rect = snap.overlay_rect.expect("docked modal should have a rect");
    let buf = terminal.backend().buffer();
    assert_eq!(buf[(rect.x, rect.y)].symbol(), "╔");
    assert_eq!(buf[(rect.x + rect.width - 1, rect.y)].symbol(), "╗");
    assert_eq!(buf[(rect.x, rect.y + rect.height - 1)].symbol(), "╚");
    assert_eq!(
        buf[(rect.x + rect.width - 1, rect.y + rect.height - 1)].symbol(),
        "╝"
    );
}
