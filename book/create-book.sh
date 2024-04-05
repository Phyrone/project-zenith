#!/usr/bin/env bash

# optional parameter to specify the format of the output
# default is html
FORMAT=${1:-html}

generate_html() {
    echo "Creating HTML book..."
   asciidoctor -t book.adoc
}

generate_pdf() {
    echo "Creating PDF book..."
    asciidoctor-pdf -t -a pdf-stylesdir=resources/themes -a pdf-style=custom -a pdf-fontsdir=resources/fonts book.adoc
}

# generate the book
case $FORMAT in
  html)
    generate_html
    ;;
  pdf)
    generate_pdf
    ;;
  '*')
    echo "Invalid format: $FORMAT"
    exit 1
    ;;
esac