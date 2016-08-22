# Kosbook user documentation

## Usage and options

    Usage: kosbook [options]

    Options:
        -h, --help          print help message and exit
        -i, --input FILE    specify input structure file (default:
                            ./structure.json)
        -o, --output FILE   specify output file (default: ./output.html)
        -p, --pdf           also invoke 'wkhtmltopdf' to produce a pdf. Note
                            that wkhtmltopdf must be in your PATH.
        -r, --rules FILE    specify the processing rules file (default:
                            ./rules.json)
        -s, --style FILE    specify custom path to CSS file (default: style.css)
        -v, --version       print version and exit

    All paths in the structure file are relative to the directory the program is
    invoked in.


## Project structure

A **Kosbook** project comprises of a _structure file_ (usually named
`structure.json`), a _rules file_ (usually named `rules.json`), a _CSS
stylesheet_ (usually named `style.css`), and any number of CommonMark files.

The _structure file_ generally lives at the top level of a book project. This is
not necessary, but it is the most intuitive option. The tool will consider all
paths in the structure file as relative to the location the tool is run from,
not relative to the _structure file_ itself; therefore it makes sense to run the
tool in the same directory where the _structure file_ is.

The typical structure of a project is:

```
Top directory
|
+-- structure.json
|
+-- rules.json
|
+-- style.css
|
+-- src
    |
    +-- CommonMark files
```

With this setup, it is possible to just run `kosbook` in the top level directory
without any option.

### A note on the HTML and CSS stylesheet

The output HTML will contain a reference to the _CSS stylesheet_ and it will
assume that the CSS file lives in the same directory of the HTML. When
distributing or deploying the output HTML, don't forget to include the
_CSS stylesheet_ next to it.


## Description of the tool operation

The tool reads the _structure file_ which contains the structure of the book,
which is divided in parts and chapters. Each chapter can have more than one
CommonMark file. The tool opens and reads all those files and creates a copy of
their content in memory. See [The structure file](#struct_file) for a
specification of how the parts and chapters are created.

Then, the tool applies the text substitution rules specified in the
_rules file_. These are essentially match/replace rules based on regular
expressions, however they are also used to match some data and store it in
variables. See [The rules file](#rules_file) for a specification of the effect
of the possible rules.

Rules are run sequentially in the order they are written in the _rules file_.

Storage variables are created by the rules, and can be of four types.

- _Single variables_ hold just one value as a string. Subsequent writes to the
  same variable of this type will overwrite the value.
- _Vector variables_ hold a list of string values. Subsequent writes to the
  same variable of this type will append values to the list.
- _Map-of-single variables_ hold string values indexed by a string key.
  Subsequent writes to the same variable of this type, with the same key, will
  overwrite the value indexed by the key.
- _Map-of-vector variables_ hold lists of string values, each list indexed by a
  string key. Subsequent writes to the same variable of this type, with the same
  key, will append values to the list indexed by the key.

After all the rules have run (and storage variables have been created), the tool
substitutes the values of the storage variables in the CommonMark content.

The CommonMark content coming from each file can contain expressions of the
type:

    {{ identifier.key }}

or just:

    {{ identifier }}

The `identifier` and the `key` shall only be comprised of underscores, digits,
and ASCII letters.

In the first case, the variable is either a _map-of-single variable_ or a
_map-of-vector variable_. The `identifier` is used to look up the map, and the
`key` is used to index in the map to extract a _single_ or _vector_ value,
respectively.

In the second case, the `identifier` is just used to look up the variable, which
can be a _single_ or _vector_.

In either case, if the looked up value is a _single_, its string value is
substituted to the whole expression (including the double braces).

If instead the looked up value is a _vector_, the expression (including the
double braces) is substituted with the entire list of string values, with values
being separated by two newline characters (`\n\n`).

Once all variable substitutions have happened, the tool proceeds to collate all
content into one document and renders it as HTML. The `style.css` file is
referenced (linked) in the HTML, and it is assumed to be in the same directory
as the HTML file produced.
If PDF output is also enabled, `wkhtmltopdf` is invoked as a sub-process
and a PDF document is created from the HTML.

The output files are normally `output.html` and `output.pdf`, but they can be
changed with a command-line option (the PDF file name is always the same as the
HTML file name, just with a different extension).


## <a id="struct_file">The structure file</a>

### Syntax

The _structure file_ shall be valid JSON and contain one JSON object at the top
level.

The top level JSON object shall contain a "title" field, an "author" field, and
a "license" field. These fields shall have string values.

The top level JSON object shall also contain a "parts" field, which is a JSON
array of JSON objects.

Each object in the "parts" array shall have a "title" field with a string value,
and a "chapters" field which is a JSON array of JSON objects.

Each object in the "chapters" array shall have a "title" field with a string
value, and a "files" field which is a JSON array of strings. Each of such
strings shall be a relative path to a CommonMark file.

### Effect

The beginning of the book shall contain the information provided in the "title",
"author" and "license" fields of the top level JSON object in the _structure
file_.

The tool shall effect this by creating some CommonMark content preceding any
content in the book.

This CommonMark content shall contain a `<div class="book_author">` HTML element
wrapping the text in the "author" string, followed by a
`<div class="book_title">` HTML element wrapping the text in the "title" string,
followed by a `<div class="book_license">` HTML element wrapping the text in the
"license" field.

The output book shall then feature a table of contents, automatically generated
from the structure. This TOC shall be CommonMark content wrapped by a
`<div class="toc">` HTML element, and containing an unordered list of parts,
each list item also having a sub-list of chapters. The text of each list item
shall contain the "title" field for the corresponding part JSON object or
chapter JSON object.

There shall be hyperlinks from each list item to the corresponding part or
chapter in the book. Links and anchors are automatically generated.

The output book shall feature parts. Each part shall start with a header, which
is generated CommonMark content. This header shall first open a
`<div class="part_N">` HTML tag, where `N` is the part index; then it shall
contain a `<div class="part_title">` HTML element wrapping the text specified in
the "title" field of the corresponding part JSON object. This text shall be the
anchor to which the TOC part links link to.

Each part header shall be followed by the content making up the chapters within
that part, as specified by the chapter JSON object contained by the part JSON
object.

Each chapter shall have an initial header, which is generated CommonMark content
featuring a first-level heading whose text shall be the text in the "title"
field of the corresponding chapter JSON object. The heading shall be the anchor
to which the TOC chapter links link to.

Each chapter shall then have the concatenation of the CommonMark content in all
the files referenced by the corresponding chapter JSON object, in the "files"
JSON array.

Finally, each part shall contain a `</div>` closing element corresponding to the
`<div class="part_N">` tag opened in the header.

### Example of the syntax

    {
        "title": "Book Title",
        "author": "Dario Domizioli",
        "license": "Licensed under the Apache License Version 2.0",
        "parts": [{
            "title": "Part I",
            "chapters": [{
                "title": "Chapter A",
                "files": ["chapterA.md", "chapterA_addendum.md"]
            }]
        }, {
            "title": "Part II",
            "chapters": [{
                "title": "Chapter B",
                "files": ["chapterB.md", "oh_and_one_last_thing.md"]
            }]
        }]
    }

Also have a look at `examples/trivial/structure.json` for another example.


## <a id="rules_file">The rules file</a>

### Syntax

The _rules file_ shall be valid JSON and contain one JSON object at the top
level.

The top level object shall contain a "rules" field, which is a JSON array of
JSON object, each denoting a _rule_.

Each _rule_ JSON object shall contain a "name" field, which can be any string.

It shall contain a "regex" field, which is a string containing a regular
expression. Note that because the format is JSON, characters in this string may
need to be escaped with the appropriate number of backslashes.

It shall also contain a "replace" field, which is a string which may contain
references to regular expression groups (e.g. `$1`).

Finally, it shall contain a "storage" field, which is an array of JSON object,
each object being a _storage spec_.

Each _storage spec_ object shall contain an "action" field, which shall be one
of the following strings:

- `StoreSingle`
- `StoreVector`
- `StoreMapSingle`
- `StoreMapVector`

The _storage spec_ shall also contain a "replace" field, which is a string which
may contain references to regular expression groups (e.g. `$1`). _This field may
be different from the "replace" field of the rule JSON object_, and in fact it
is often useful for it to be different.

The _storage spec_ shall also contain a "variable" field which is a string made
only of underscores, digits, and ASCII letters.

Finally, the _storage spec_ shall contain a "key" field which can either be an
empty string or a string which may contain references to regular expression
groups (e.g. `$1`). The result of substituting regular expression groups in the
string MUST produce a result which is only made of underscores, digits, and
ASCII letters.

_(As a suggestion, make sure that the groups used in the "key" string are
matched using something like "[\_0-9a-zA-Z]+" to avoid any possibility of stray
characters being substituted)_

### Effect

Each _rule_ specifies a regular expression that shall be matched in the
CommonMark content of all the chapters in the book. This regular expression is
the "regex" field.

Each match of the regular expression shall be processed as follows.

First, each _storage spec_ is processed.

If the _storage spec_'s "action" field is `StoreSingle`, then:
- The variable named as specified by the "variable" field shall be written.
- The value written shall be the result of substituting the matched regular
  expression groups in the "replace" string of the _storage spec_.

If the _storage spec_'s "action" field is `StoreVector`, then:
- The list variable named as specified by the "variable" field shall be appended
  to.
- The value appended to the list shall be the result of substituting the matched
  regular expression groups in the "replace" string of the _storage spec_.

If the _storage spec_'s "action" field is `StoreMapSingle`, then:
- The variable named as specified by the "variable" field shall be indexed into,
  with a key equal to the result of substituting the matched regular expression
  groups in the "key" string.
- The indexed value shall be written.
- The value written shall be the result of substituting the matched regular
  expression groups in the "replace" string of the _storage spec_.

If the _storage spec_'s "action" field is `StoreMapVector`, then:
- The variable named as specified by the "variable" field shall be indexed into,
  with a key equal to the result of substituting the matched regular expression
  groups in the "key" string.
- The indexed list shall be appended to.
- The value appended to the list shall be the result of substituting the matched
  regular expression groups in the "replace" string of the _storage spec_.

Then, the text which was matched in the CommonMark content is replaced in-place
with the result of substituting the matched regular expression groups in the
"replace" string of the _rule_.

### Examples of the syntax

Just substitute some text, no storage:

```
{
    "rules": [{
        "name": "Bring text out of a comment",
        "regex": "<!--\\s*(.*)\\s*-->",
        "replace": "$1",
        "storage": []
    }]
}
```

This rule stores key-value pairs but removes the comment entirely from the
content, due to the empty "replace" string:

```
{
    "rules": [{
        "name": "Define key-value pairs",
        "regex": "<!--\\s*key=\"([_0-9a-zA-Z]+)\"\\s*value=\"(.*)\"\\s*-->",
        "replace": "",
        "storage": [{
            "action": "StoreMapSingle",
            "replace": "$2",
            "key": "$1",
            "variable": "my_little_map"
        }]
    }]
}
```

## Styling the output book

There are only a few HTML elements that need to be styled in the output book.
This is because the output layout is intentionally simple.

The cover of the book needs styling for the following elements:

- `div.book_author`
- `div.book_title`
- `div.book_license`

The table of contents needs styling for the following elements:

- `div.toc`
- `.toc ul li` (for the part headings)
- `.toc ul li ul li` (for the chapter headings)


Individual parts are contained within `div.part_N` elements (with N being the
part index) which need to be individually styled, although part titles are
always the same and styled with the `div.part_title` element. This allows for
example to have a different background image or color for each part, but to
maintain the same font style for the part titles.

Chapter titles are `h1` elements and need to be styled as such.

Of course, any other element can be freely styled as required.
