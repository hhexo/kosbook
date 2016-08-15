# Kosbook

**Kosbook** is an authoring tool designed to produce a single HTML (or PDF)
document out of a collection of [CommonMark](http://http://commonmark.org/)
files. It is written in [Rust](http://rust-lang.org), based on the excellent
[pulldown-cmark](https://github.com/google/pulldown-cmark) Rust crate, and
inspired by [GitBook](http://gitbook.com).

**Kosbook** is licensed under the Apache License Version 2.0, a permissive
license. See the LICENSE file for details.

## Why not just use GitBook?

By all means, do.

My motivation for writing my own tool comes from the fact that GitBook, if
anything, does _too much_. All I needed was a tool to collect some CommonMark
files, do some simple text substitutions (e.g. handle variables), glue the files
up in a single document, and create an HTML or PDF file with my own CSS style.

GitBook can do that, and much more thanks to its plugin system, but I have two
main issues with it.

One is the layout. GitBook creates HTML with a very good layout which is however
quite complicated; it includes a search function, collapsable panes, and so on.
If you want to style it with CSS, you have to apply the styles to elements that
are five or six levels deep in `<div>`s, and if you want to change the HTML
layout, you have to write a theme plugin (and if you want to reuse it in
different projects you probably want to publish it in `npm`)... it's possible,
but it takes work.

The second one is the PDF conversion. GitBook uses `ebook-convert`, available
with [Calibre](https://calibre-ebook.com), but I have had issues with it - it
ignores some of my print CSS styles, and I'm not happy about some of its
heuristics. I have had a better experience with the open-source
[wkhtmltopdf](http://wkhtmltopdf.org).

So, don't get me wrong. GitBook is a lovely product. It just doesn't do exactly
what I need, and rather than spending a lot of time customizing its layout and
PDF options, I found that I could actually write my own tool in a shorter time.
Version 0.1.0 of **Kosbook** was written in a weekend.

## Building Kosbook

**Kosbook** is a Cargo project, so

    cargo build

is enough to build a binary. You can then put it somewhere on your `PATH`.

## Using Kosbook

Using **Kosbook** only requires you to write your CommonMark files (in whatever
directory structure you like) and a couple of JSON files.

If you want to create PDFs, you will have to have
[wkhtmltopdf](http://wkhtmltopdf.org) installed and on your `PATH`.

A more detailed user documentation is available in `DOCS.md` in this repository,
but here is a brief summary.

Your book project will usually look like this:

```
Top directory
|
+-- structure.json
|
+-- rules.json
|
+-- style.css
|
+-- Whatever CommonMark files you want to have
|
+-- Subfolders
    |
    +-- Whatever CommonMark files you want to have
```

For most projects, you just need to run `kosbook` in the top level directory of
your book project.

The tool reads `structure.json` which contains the structure of your book,
which is divided in parts and chapters. Each chapter can have more than one
CommonMark file (they are just concatenated in the order you specify them). The
tool opens and reads all those files and creates a copy of the content in
memory.

The syntax of `structure.json` is quite simple and described in the
documentation. Also have a look at `examples/trivial/structure.json` for an
example.

Then, the tool applies the text substitution rules specified in `rules.json`.
These are essentially match/replace rules based on regular expressions, however
they can also be used to match some data and store it in variables.

The `rules.json` file is essentially a domain-specific language implemented in
JSON, but the syntax is again reasonably easy to use. Have a look at
`examples/trivial/rules.json` for a very easy "search and replace" rule, and
read the documentation for further details.

After all the rules have run (and variables have been created), the tool
substitutes the values of the variables in the CommonMark content. Variables are
referenced with the commonly used notation `{{ variable_name }}`.

Finally, the content is collated in a single big document, and it is rendered to
HTML. The `style.css` file is referenced in the HTML, and you can put all your
custom styling in there; there are minimal hierarchy levels enforced by the
tool in the layout, so creating a CSS style is going to be easy.
If PDF output is also enabled, `wkhtmltopdf` is invoked as a sub-process
and a PDF document is created from the HTML.

The output files are normally `output.html` and `output.pdf`, but they can be
changed with a command-line option (the PDF file name is always the same as the
HTML file name, just with a different extension).

## Examples

Examples live in the `examples/` directory. At the moment there is only a
trivial example.

## FAQ

### Why is it named Kosbook?

Well... I thought about writing this tool while I was writing a role playing
games rulebook, and the game is called Kosben. So this is the tool I use for the
Kosben Book... **Kosbook**.

### Why is the structure in JSON rather than CommonMark (like in GitBook)?

Writing the table of contents in CommonMark, with a list of links, forces you to
have _one file per chapter_. My chapters are quite long and I want to split
them over multiple files; with a JSON structure, I can have a JSON array of
files for each chapter.

Yes, it's an unusual use case, but that's my reason.

### Can I change the PDF page size and margin?

Not at the moment. I'm definitely planning to add more customization features in
further versions.

### Ah. So how ready _really_ is Kosbook?

Well, this is version `0.1.0`. It does what I want already, but there's a lot of
stuff that can be improved.

### Can I contribute?

I'm still tinkering with **Kosbook**, so it might be that your pull requests
will often need to be rebased. But sure, pull requests are welcome.

### Why do I have to install `wkhtmltopdf`? Can't you include it in Kosbook?

Not if I want to keep the entire **Kosbook** product Apache-licensed, just like
Rust. You see, `wkhtmltopdf` is LGPL-licensed. I know, the open-source licensing
wars are annoying, but I prefer Apache, and that's it.
