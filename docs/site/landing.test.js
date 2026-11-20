'use strict';

const assert = require('node:assert/strict');
const { pickLanguage, browserLanguages, LANGUAGES } = require('./landing.js');

assert.equal(pickLanguage(['pl-PL', 'en-US']), 'pl');
assert.equal(pickLanguage(['pl']), 'pl');
assert.equal(pickLanguage(['en-GB', 'pl']), 'en');
assert.equal(pickLanguage(['ja', 'de-AT', 'pl']), 'de');
assert.equal(pickLanguage(['pt-PT']), 'pt-br');
assert.equal(pickLanguage(['pt-BR']), 'pt-br');
assert.equal(pickLanguage(['UK']), 'uk');
assert.equal(pickLanguage(['uk_UA']), 'uk');
assert.equal(pickLanguage(['ja', 'zh-CN']), 'en');
assert.equal(pickLanguage([]), 'en');
assert.equal(pickLanguage(undefined), 'en');

assert.deepEqual(browserLanguages({ languages: ['pl-PL', 'en'], language: 'en' }), ['pl-PL', 'en']);
assert.deepEqual(browserLanguages({ language: 'fr-FR' }), ['fr-FR']);
assert.deepEqual(browserLanguages({}), []);

const codes = ['en', 'pl', 'de', 'es', 'fr', 'it', 'nl', 'pt-br', 'uk'];
assert.deepEqual(Object.keys(LANGUAGES).sort(), codes.slice().sort());
for (const code of codes) {
  const entry = LANGUAGES[code];
  assert.ok(entry.description.length > 20, code);
  assert.equal(entry.links.length, 4, code);
  assert.equal(entry.pages.length, 4, code);
}

function fakeElement(tag) {
  const element = {
    tag, children: [], attrs: {}, classes: new Set(), href: '', lang: '', className: '', textContent: '',
    get firstChild() { return this.children[0] || null; },
    appendChild(child) { this.children.push(child); },
    removeChild(child) { this.children.splice(this.children.indexOf(child), 1); },
    setAttribute(name, value) { this.attrs[name] = value; },
    removeAttribute(name) { delete this.attrs[name]; },
    getAttribute(name) { return this.attrs[name]; },
    classList: { toggle(name, on) { on ? element.classes.add(name) : element.classes.delete(name); } },
  };
  return element;
}

const centre = fakeElement('div');
const flags = ['en', 'pl'].map((code) => {
  const anchor = fakeElement('a');
  anchor.attrs['data-code'] = code;
  return anchor;
});
const fakeDoc = {
  getElementById: (id) => (id === 'lp-centre' ? centre : null),
  createElement: fakeElement,
  querySelectorAll: () => flags,
};
const { render } = require('./landing.js');
const polish = pickLanguage(browserLanguages({ languages: ['pl-PL', 'pl', 'en-US'] }));
render(fakeDoc, polish);
assert.equal(centre.children[0].textContent, LANGUAGES.pl.description);
assert.ok(centre.children[0].textContent.startsWith('Pasjans w terminalu'));
assert.equal(centre.children[1].children[0].children[0].href, 'pl/index.html');
assert.ok(flags[1].classes.has('lp-current') && flags[1].attrs['aria-current'] === 'true');
assert.ok(!flags[0].classes.has('lp-current') && flags[0].attrs['aria-current'] === undefined);

console.log('landing.js: all checks passed');
