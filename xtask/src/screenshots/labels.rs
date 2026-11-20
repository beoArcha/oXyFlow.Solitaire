//! Alternative text of every image, in the site languages.

use oxysol_tui::i18n::Locale;

const LABELS: [(&str, &str, &str); 16] = [
    ("menu", "The oXySol main menu", "Menu główne oXySol"),
    (
        "klondike",
        "Klondike with the guide and the possible moves panels beside the board",
        "Klondike z panelem poradnika i panelem możliwych ruchów obok planszy",
    ),
    (
        "spider",
        "Spider with two suits",
        "Pająk z dwoma kolorami kart",
    ),
    ("freecell", "FreeCell", "FreeCell"),
    ("pyramid", "Pyramid", "Piramida"),
    ("settings", "The settings screen", "Ekran ustawień"),
    (
        "help",
        "The help window listing every key",
        "Okno pomocy z listą wszystkich klawiszy",
    ),
    (
        "guide",
        "A hint marked on the board, explained in the guide panel",
        "Podpowiedź zaznaczona na planszy i opisana w panelu poradnika",
    ),
    ("theme-dark", "The Dark theme", "Motyw Ciemny"),
    ("theme-light", "The Light theme", "Motyw Jasny"),
    (
        "theme-high-contrast",
        "The High contrast theme",
        "Motyw Wysoki kontrast",
    ),
    ("theme-monokai", "The Monokai theme", "Motyw Monokai"),
    ("theme-dracula", "The Dracula theme", "Motyw Dracula"),
    (
        "anim-move",
        "Animation: the cursor picks up the ace of clubs and drops it on a foundation",
        "Animacja: kursor podnosi asa trefl i kładzie go na stosie końcowym",
    ),
    (
        "anim-auto-finish",
        "Animation: auto-finish plays out a won game move by move",
        "Animacja: automatyczne kończenie rozgrywa wygraną partię ruch po ruchu",
    ),
    (
        "anim-win",
        "Animation: the last moves of a won game and the win window",
        "Animacja: ostatnie ruchy wygranej partii i okno wygranej",
    ),
];

pub fn label(name: &str, locale: Locale) -> &'static str {
    LABELS
        .iter()
        .find(|(known, _, _)| *known == name)
        .map_or("", |(_, english, polish)| match locale {
            Locale::PlPl => polish,
            _ => english,
        })
}
