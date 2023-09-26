module.exports = grammar({
    name: 'Stilts',
    rules: {
        source_file: $ => repeat(choice($.expr, $.text)),
        expr: $ => seq('{%', optional($.expr_content), '%}'),
        expr_content: $ => /([^%]|[%][^}])*/, // TODO: support %} within rust strings like the language does (probably requires a scanner.c)
        text: $ => /([^{]|[{][^%])*/,
    }
})
