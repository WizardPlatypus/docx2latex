# docx2latex

A command line utility that converts docx files into latex templates.

# Usage

```
docx2latex <input.docx> <output-folder>
```

# Explanation

The program will unzip the ooxml package that is the docx file, identify useful files within that package, build an XML tree, and convert every node it can into a latex expression.