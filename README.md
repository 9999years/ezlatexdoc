# ezlatexdoc

I don't want to learn how to use docstrip and docstrip source looks ugly anyways.

Here's the rules:
1. If the line starts with `%%%`, it's an ezlatexdoc directive; ignored for
   now, maybe they'll do something in the future.
2. If the line starts with `%%`, it's a comment kept in the resulting source.
3. If the line starts with `%`, it's a doc comment, written to the doc file.
4. If a line contains an ending comment, the comment's *content* is stripped
   out but the comment marker is kept in, because LaTeX is weird and I don't
   want to implement a parser. (Well, at least not as part of *this*
   project...)
