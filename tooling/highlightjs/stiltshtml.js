/*
Language: Stiltshtml
Author: Jack Morrison <jackamorr@gmail.com>
Website: https://atrociously.github.io/stilts
Category: web, template
*/

/** @type LanguageFn */
export default function(hljs) {
  return {
    name: 'StiltsHtml',
    aliases: [ 'stilts' ],
    subLanguage: 'xml',
    contains: [
      {
        scope: "expression",
        begin: /{%/, end: /%}/,
        beginScope: "delim-open",
        endScope: "delim-close",
        subLanguage: "rust"
      }
    ]
  };
}
