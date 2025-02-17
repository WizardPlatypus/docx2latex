# docx2latex

A command line utility that converts docx files into latex templates.

# Usage

```
docx2latex <input.docx> <output-folder>
```

# Explanation

The program will unzip the ooxml package that is the docx file, identify useful files within that package, build an XML tree, and convert every node it can into a latex expression.

# Notes

## Plain text
Done yay

## Hyperlinks

There are a few different ways that Word works with hyperlinks. According to this piece of [documentation](http://officeopenxml.com/WPhyperlink.php), OOXML uses relationship ids for external resources, and anchors for internal links. This way, information necessary for parsing the hyperlink is gathered in one place. However, the version of Office installed on my machine in particular, uses scripts to display hyperlinks when tasked to do so "in place". Scripts are beyond the scope of this project, so we will ignore this usecase for now.