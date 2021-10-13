// Syntax highlighting for erd-script.

hljs.registerLanguage("erd", function (hljs) {
  // Basic syntax.
  var comment = { className: "comment", begin: "//", end: /$/ };
  var ident = {
    className: "title",
    begin: /[_a-zA-Z][_a-z0-9A-Z]*/,
    end: /$/,
  };
  var special = {
    begin:
      /entity|attribute|id|relation|one|exactly|multiple|required|optional|table|from|type/,
    className: "keyword",
  };

  // Escape sequences within a string or character literal.
  var escape = { begin: /\\./ };

  var string_ident = {
    begin: /[_a-zA-Z][_a-z0-9A-Z]*/,
    end: /$/,
    contains: [
      { className: "string", begin: /\(/, end: /\)/ }, //, contains: [escape] },
    ],
  };

  return {
    contains: [special, string_ident, ident, comment],
  };
});

// This file is inserted after the default highlight.js invocation, which tags
// unknown-language blocks with CSS classes but doesn't highlight them.
Array.from(document.querySelectorAll("code.language-erd")).forEach(
  hljs.highlightBlock
);
