(function (root) {
  'use strict';

  var DEFAULT_LANGUAGE = 'en';

  var LANGUAGES = {
    en: {
      description: 'A terminal solitaire: Klondike, Spider, FreeCell and Pyramid in one program, the single binary oxysol. It runs in any modern terminal and is played with the keyboard or the mouse.',
      links: ['Introduction', 'Features', 'Screenshots and animations', 'Install'],
      pages: ['en/index.html', 'features.html', 'gallery.html', 'install.html']
    },
    pl: {
      description: 'Pasjans w terminalu: Klondike, Pająk, FreeCell i Piramida w jednym programie — pojedynczym pliku oxysol. Działa w każdym nowoczesnym terminalu, a grać można klawiaturą lub myszą.',
      links: ['Wprowadzenie', 'Funkcje', 'Zrzuty ekranu i animacje', 'Instalacja'],
      pages: ['pl/index.html', 'pl/features.html', 'pl/gallery.html', 'pl/install.html']
    },
    de: {
      description: 'Solitär im Terminal: Klondike, Spider, FreeCell und Pyramide in einem Programm, einer einzigen Datei: oxysol. Es läuft in jedem modernen Terminal und wird mit der Tastatur oder der Maus gespielt.',
      links: ['Einführung', 'Funktionen', 'Screenshots und Animationen', 'Installation'],
      pages: ['de/index.html', 'de/features.html', 'de/gallery.html', 'de/install.html']
    },
    es: {
      description: 'Solitario en la terminal: Klondike, Spider, FreeCell y Pirámide en un solo programa, un único archivo: oxysol. Funciona en cualquier terminal moderna y se juega con el teclado o el ratón.',
      links: ['Introducción', 'Funciones', 'Capturas de pantalla y animaciones', 'Instalación'],
      pages: ['es/index.html', 'es/features.html', 'es/gallery.html', 'es/install.html']
    },
    fr: {
      description: 'Le solitaire dans votre terminal : Klondike, Spider, FreeCell et Pyramide réunis dans un seul programme, un fichier unique : oxysol. Il fonctionne dans tout terminal moderne et se joue au clavier ou à la souris.',
      links: ['Introduction', 'Fonctionnalités', 'Captures d’écran et animations', 'Installation'],
      pages: ['fr/index.html', 'fr/features.html', 'fr/gallery.html', 'fr/install.html']
    },
    it: {
      description: 'Il solitario nel terminale: Klondike, Spider, FreeCell e Piramide in un unico programma, un solo file: oxysol. Funziona in qualsiasi terminale moderno e si gioca con la tastiera o con il mouse.',
      links: ['Introduzione', 'Funzionalità', 'Screenshot e animazioni', 'Installazione'],
      pages: ['it/index.html', 'it/features.html', 'it/gallery.html', 'it/install.html']
    },
    nl: {
      description: 'Patience in je terminal: Klondike, Spider, FreeCell en Piramide in één programma — één enkel bestand: oxysol. Het werkt in elke moderne terminal en je speelt het met het toetsenbord of de muis.',
      links: ['Inleiding', 'Functies', 'Schermafbeeldingen en animaties', 'Installeren'],
      pages: ['nl/index.html', 'nl/features.html', 'nl/gallery.html', 'nl/install.html']
    },
    'pt-br': {
      description: 'Paciência no terminal: Klondike, Spider, FreeCell e Pirâmide em um único programa, um só arquivo: oxysol. Funciona em qualquer terminal moderno e pode ser jogado com teclado ou mouse.',
      links: ['Introdução', 'Recursos', 'Capturas de tela e animações', 'Instalação'],
      pages: ['pt-br/index.html', 'pt-br/features.html', 'pt-br/gallery.html', 'pt-br/install.html']
    },
    uk: {
      description: 'Пасьянс у терміналі: Klondike, Павук, FreeCell і Піраміда в одній програмі — єдиному файлі oxysol. Працює в будь-якому сучасному терміналі, грати можна клавіатурою або мишею.',
      links: ['Вступ', 'Можливості', 'Знімки екрана та анімації', 'Встановлення'],
      pages: ['uk/index.html', 'uk/features.html', 'uk/gallery.html', 'uk/install.html']
    }
  };

  var BY_LANGUAGE_PART = { en: 'en', pl: 'pl', de: 'de', es: 'es', fr: 'fr', it: 'it', nl: 'nl', pt: 'pt-br', uk: 'uk' };

  function pickLanguage(languages) {
    var list = languages || [];
    for (var i = 0; i < list.length; i += 1) {
      var part = String(list[i]).toLowerCase().split(/[-_]/)[0];
      if (Object.prototype.hasOwnProperty.call(BY_LANGUAGE_PART, part)) {
        return BY_LANGUAGE_PART[part];
      }
    }
    return DEFAULT_LANGUAGE;
  }

  function browserLanguages(nav) {
    if (nav && nav.languages && nav.languages.length) {
      return nav.languages;
    }
    return nav && nav.language ? [nav.language] : [];
  }

  function render(doc, code) {
    var centre = doc.getElementById('lp-centre');
    if (!centre) {
      return;
    }
    var data = LANGUAGES[code];
    var description = doc.createElement('p');
    description.className = 'lp-desc';
    description.lang = code === 'pt-br' ? 'pt-BR' : code;
    description.textContent = data.description;
    var list = doc.createElement('ul');
    list.className = 'lp-links';
    list.lang = description.lang;
    data.links.forEach(function (label, index) {
      var item = doc.createElement('li');
      var anchor = doc.createElement('a');
      anchor.href = data.pages[index];
      anchor.textContent = label;
      item.appendChild(anchor);
      list.appendChild(item);
    });
    while (centre.firstChild) {
      centre.removeChild(centre.firstChild);
    }
    centre.appendChild(description);
    centre.appendChild(list);
    var flags = doc.querySelectorAll('.lp-flags a[hreflang]');
    Array.prototype.forEach.call(flags, function (anchor) {
      var current = anchor.getAttribute('data-code') === code;
      anchor.classList.toggle('lp-current', current);
      if (current) {
        anchor.setAttribute('aria-current', 'true');
      } else {
        anchor.removeAttribute('aria-current');
      }
    });
  }

  var api = { pickLanguage: pickLanguage, browserLanguages: browserLanguages, render: render, LANGUAGES: LANGUAGES };

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api;
  }
  if (root.document && root.document.getElementById('lp-centre')) {
    render(root.document, pickLanguage(browserLanguages(root.navigator)));
  }
})(typeof window !== 'undefined' ? window : {});
