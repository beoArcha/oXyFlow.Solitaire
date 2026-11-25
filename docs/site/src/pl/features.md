[English](../features.md) · **Polski** · [Deutsch](../de/features.md) · [Español](../es/features.md) · [Français](../fr/features.md) · [Italiano](../it/features.md) · [Nederlands](../nl/features.md) · [Português (Brasil)](../pt-br/features.md) · [Українська](../uk/features.md)

# Funkcje

## Gry

Czternaście gier w sześciu rodzinach, jeden program, jeden zestaw klawiszy:

- **Klondike**: dobieranie 1 karty albo 3 kart.
- **Pająk**: 1, 2 lub 4 kolory.
- **FreeCell**.
- **Piramida**: swobodna albo klasyczna.
- **Rodzina Yukon**: Yukon, Rosyjski pasjans, Alaska, Moosehide i Australijski.
- **Golf**: opróżnianie siedmiu kolumn na stos odrzuconych, po jednej randze w górę lub w dół naraz.

Menu gier grupuje gry pod nagłówkami rodzin, oznacza gwiazdką te popularne, trzyma ulubione na osobnej liście i znajduje grę, kiedy wpisujesz jej nazwę. Zasady każdej gry są w oknie pomocy (`i`, „Jak grać”).

## Rozdania: ziarna, rozdania do wygrania i poziomy trudności

Każde rozdanie ma **ziarno** (seed). Skopiuj je klawiszem `c`, żeby powtórzyć rozdanie albo się nim podzielić.

- **Tylko rozdania do wygrania.** Nowa gra może poczekać na rozdanie, które wbudowany solver udowodnił jako możliwe do wygrania. Solver zna każdą kartę, więc „do wygrania” oznacza, że rozdanie da się wygrać, a nie że jest łatwe.
- **Nowa gra z ziarna.** Najpierw wybierz grę, potem skąd ma pochodzić ziarno: wpisz je, wybierz jedno z rozdań, które już grałeś (najnowsze na początku, z wynikiem i czasem), pozwól grze wylosować jedno z nich albo weź jedno z **wyselekcjonowanych rozdań**.
- **Pięć poziomów trudności.** Każda gra oferuje po trzy wyselekcjonowane rozdania na poziom, od bardzo łatwego do bardzo trudnego. Wszystkie są udowodnione jako możliwe do wygrania. Poziomy wynikają z pomiarów: ile pracy wymagał solver, jak długa jest linia wygrywająca i czy samo podążanie za podpowiedziami wystarcza, by wygrać rozdanie.

## Pomoc podczas gry

- **Panel przewodnika** (`g`): cel gry, klawisze działające w tej chwili i to, co może zrobić karta w fokusie. Przy 130 kolumnach lub więcej stoi obok planszy, z przyciskiem **Nowa gra** u góry; na węższych terminalach otwiera się w oknie.
- **Panel możliwych ruchów** (`v`): każdy ruch, który możesz teraz wykonać, z jego klawiszem, oraz powód, gdy ruch nie jest możliwy.
- **Podpowiedzi** (`h`): gra zaznacza ruch na planszy i go nazywa. Podpowiedzi korzystają tylko z tego, co widzisz: nigdy nie zaglądają w karty zakryte ani w talię.
- **Szybkie ruchy**: `a` wysyła karty pod kursorem na najlepsze miejsce, `e` robi to samo z wierzchnią kartą stosu odrzuconych z dowolnego miejsca na planszy, a dwuklik działa jak `a`. Szybkie ruchy można wyłączyć w ustawieniach albo w szybkich opcjach (`o`), jeśli wolisz sam układać każdą kartę.
- **Automatyczne kończenie**: gdy wygrana jest pewna, pozostałe ruchy są wykonywane za Ciebie. Można to wyłączyć.
- **Cofnij i ponów** (`u`, `r`), z opcją gry bez cofania.

## Klawiatura i mysz

Każda czynność działa klawiaturą i myszą. Kliknij kartę, żeby ją podnieść, a potem cel, żeby ją położyć, zrób dwuklik dla szybkiego ruchu, przeciągnij kartę na cel albo kliknij prawym przyciskiem, żeby ją odłożyć. Przyciski w oknach i panelach pokazują swój klawisz, a aktywna zakładka okna nigdy nie wygląda jak przycisk w fokusie. Okno pomocy (`?`) wymienia każdy klawisz; może też zostać przycumowane z boku ekranu (`d`).

## Ustawienia

Ustawienia mają trzy zakładki:

- **Popularne** otwiera się pierwsza i zbiera ustawienia zmieniane najczęściej: język, tryb słabowidzący, motyw, rozmiar kart, rewers kart, głośność, aurę i automatyczne kończenie.
- **Dźwięk i wygląd** zawiera wszystko, co dotyczy wyglądu i dźwięku: kolory, pełny ekran i rozmiar okna, motyw, widzenie barw, style i odcienie kart, stół, aurę, czcionkę konsoli, dźwięk i priorytet procesu.
- **Rozgrywka** zawiera pomoce: znaczniki celów, panele przewodnika i możliwych ruchów, automatyczne kończenie i szybkie ruchy.

Podgląd obok listy pokazuje zmianę, zanim ją zachowasz.

## Karty i stół

- **Sześć stylów awersów**: Międzynarodowy, Francuski, Niemiecki, Włoski, Hiszpański i Szwajcarski, każdy z własnymi znakami kolorów.
- **Dwa style rewersów** (Kratka i Ornament) oraz kolory rewersów z motywu, Monokai lub Dracula.
- **Odcień tła karty**: z motywu, biały, kremowy, jasnoszary albo ciemny.
- **Stół**: kolor motywu, zielony filc, niebieski, czerwony, szary, brązowy lub czarny, oraz powierzchnia gładka, filcowa albo liniowana.
- **Rozmiar kart**: powiększaj karty klawiszem `]`, a pomniejszaj klawiszem `[`; `0` przywraca automatyczny rozmiar, który mieści się w oknie. Długie kolumny się zbliżają, żeby ich ostatnia karta była zawsze w całości widoczna, a karty zmniejszają się tylko wtedy, gdy samo zbliżanie nie wystarcza.

## Motywy i dostępność

Pięć motywów: **Ciemny** i **Jasny** do codziennej gry, **Wysoki kontrast** dla słabego wzroku, **Monokai** i **Dracula**. Każdy pokazano tutaj na tej samej pozycji.

![Motyw Ciemny](img/pl/theme-dark.svg)

![Motyw Jasny](img/pl/theme-light.svg)

![Motyw Wysoki kontrast](img/pl/theme-high-contrast.svg)

![Motyw Monokai](img/pl/theme-monokai.svg)

![Motyw Dracula](img/pl/theme-dracula.svg)

- **Tryb słabowidzący.** Jeden przełącznik, proponowany przy pierwszym uruchomieniu i w ustawieniach: kontrast tekstu co najmniej 7:1 w każdym motywie, największe karty, jakie się mieszczą, gruba aura, pogrubiona ranga i kolor, większe znaczniki fokusu i celów oraz, w klasycznej konsoli Windows, większa czcionka. Każdy element pozostaje do regulacji, a wyłączenie trybu przywraca wcześniejsze ustawienia.
- **Widzenie barw.** Tryby dla protanopii, deuteranopii, tritanopii i monochromatyczny zmieniają kolory tak, żeby to, co musisz rozróżniać, pozostało rozróżnialne. **Talia czterokolorowa** nadaje każdemu kolorowi własną barwę: piki czarne, kiery czerwone, karo niebieskie i trefle zielone.
- **Kolor nigdy nie jest jedynym sygnałem.** Kolory to znaki (♠ ♥ ♦ ♣) obok rangi, element w fokusie ma symbol, podniesione karty i cele mają własne znaczniki, wyniki noszą ✓ albo ✗, a linia pod planszą mówi słowami, gdzie jest kursor i co może zrobić.

## Statystyki

Ekran statystyk pokazuje wyniki każdej gry (wygrane, serie, najlepszy czas i najmniejszą liczbę ruchów) oraz historię każdego rozdania, najnowsze na początku i w Twoim czasie lokalnym. Z historii możesz ponownie zagrać ziarno, skopiować je albo poprosić solver, żeby sprawdził, czy rozdanie było możliwe do wygrania.

## Konsola Windows

W Windows gra uruchomiona dwukrotnym kliknięciem albo poleceniem `oxysol -n` ma własne okno konsoli. Domyślnie otwiera się na pełnym ekranie, a w trybie okienkowym jej rozmiar wybierzesz w ustawieniach: 130 × 40, 195 × 60 lub 260 × 80. Gra uruchomiona w już otwartej konsoli zostawia tę konsolę bez zmian. W klasycznej konsoli Windows Ctrl+Plus, Ctrl+Minus i Ctrl+0 zmieniają rozmiar czcionki konsoli; inne terminale powiększają klawiszami, które mają własne.

## Języki

Gra jest dostępna w dziewięciu językach: angielskim, polskim, niemieckim, hiszpańskim, francuskim, włoskim, niderlandzkim, portugalskim brazylijskim i ukraińskim. Wybierz jeden z selektora flag w lewym górnym rogu menu głównego albo w ustawieniach; zmiana działa od razu. Przy pierwszym uruchomieniu gra idzie za językiem systemu. Ta strona jest dostępna w tych samych dziewięciu językach.

## Czego jeszcze nie ma

oXySol jest wersją przedpremierową. Spodziewaj się nierówności i zgłaszaj je w
[rejestrze zgłoszeń](https://github.com/beoArcha/oXyFlow.Solitaire/issues).
