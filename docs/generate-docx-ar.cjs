const fs = require("fs");
const {
  Document,
  Packer,
  Paragraph,
  TextRun,
  HeadingLevel,
  Table,
  TableRow,
  TableCell,
  WidthType,
  AlignmentType,
  BorderStyle,
  ShadingType,
  PageNumber,
  NumberFormat,
  Header,
  Footer,
  TableOfContents,
  LevelFormat,
  convertInchesToTwip,
  PageBreak,
  TabStopPosition,
  TabStopType,
  UnderlineType,
  ExternalHyperlink,
} = require("docx");

const mdContent = fs.readFileSync(
  "E:\\مشاريع كلاود\\Constellation\\docs\\help.ar\\User Manual.md",
  "utf-8"
);

const BLUE = "2E75B6";
const DARK_BLUE = "1F4E79";
const LIGHT_GRAY = "F2F2F2";
const WHITE = "FFFFFF";
const BLACK = "000000";
const FONT = "Arial";

// Parse markdown into blocks
function parseMarkdown(md) {
  const lines = md.split("\n");
  const blocks = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    // Skip horizontal rules
    if (/^---\s*$/.test(line.trim())) {
      i++;
      continue;
    }

    // Skip empty lines
    if (line.trim() === "") {
      i++;
      continue;
    }

    // Code block
    if (line.trim().startsWith("```")) {
      const lang = line.trim().replace(/^```/, "").trim();
      i++;
      const codeLines = [];
      while (i < lines.length && !lines[i].trim().startsWith("```")) {
        codeLines.push(lines[i]);
        i++;
      }
      i++; // skip closing ```
      blocks.push({ type: "code", content: codeLines.join("\n"), lang });
      continue;
    }

    // Headings
    const h1Match = line.match(/^# (.+)/);
    if (h1Match) {
      blocks.push({ type: "h1", content: h1Match[1] });
      i++;
      continue;
    }
    const h2Match = line.match(/^## (.+)/);
    if (h2Match) {
      blocks.push({ type: "h2", content: h2Match[1] });
      i++;
      continue;
    }
    const h3Match = line.match(/^### (.+)/);
    if (h3Match) {
      blocks.push({ type: "h3", content: h3Match[1] });
      i++;
      continue;
    }
    const h4Match = line.match(/^#### (.+)/);
    if (h4Match) {
      blocks.push({ type: "h4", content: h4Match[1] });
      i++;
      continue;
    }

    // Table
    if (line.includes("|") && i + 1 < lines.length && /^\|[-| :]+\|$/.test(lines[i + 1].trim())) {
      const tableRows = [];
      // Header row
      tableRows.push(
        line
          .trim()
          .replace(/^\|/, "")
          .replace(/\|$/, "")
          .split("|")
          .map((c) => c.trim())
      );
      i++; // skip header
      i++; // skip separator
      while (i < lines.length && lines[i].trim().startsWith("|")) {
        tableRows.push(
          lines[i]
            .trim()
            .replace(/^\|/, "")
            .replace(/\|$/, "")
            .split("|")
            .map((c) => c.trim())
        );
        i++;
      }
      blocks.push({ type: "table", rows: tableRows });
      continue;
    }

    // Numbered list
    const olMatch = line.match(/^\d+\.\s+(.+)/);
    if (olMatch) {
      const items = [];
      while (i < lines.length && /^\d+\.\s+/.test(lines[i])) {
        items.push(lines[i].replace(/^\d+\.\s+/, ""));
        i++;
      }
      blocks.push({ type: "ol", items });
      continue;
    }

    // Bullet list
    if (line.match(/^- /)) {
      const items = [];
      while (i < lines.length && lines[i].match(/^- /)) {
        items.push(lines[i].replace(/^- /, ""));
        i++;
      }
      blocks.push({ type: "ul", items });
      continue;
    }

    // TOC placeholder line (skip, we generate our own)
    if (/^\d+\.\s+\[/.test(line)) {
      i++;
      continue;
    }

    // Regular paragraph
    blocks.push({ type: "paragraph", content: line });
    i++;
  }

  return blocks;
}

// Parse inline markdown (bold, italic, code, links)
function parseInline(text) {
  const runs = [];
  // Regex for inline patterns
  const regex =
    /(\*\*(.+?)\*\*)|(`(.+?)`)|(\[(.+?)\]\((.+?)\))|(\[\[(.+?)\]\])/g;
  let lastIndex = 0;
  let match;

  while ((match = regex.exec(text)) !== null) {
    // Add plain text before match
    if (match.index > lastIndex) {
      runs.push(
        new TextRun({
          text: text.substring(lastIndex, match.index),
          font: FONT,
          size: 22,
        })
      );
    }

    if (match[1]) {
      // Bold
      runs.push(
        new TextRun({
          text: match[2],
          bold: true,
          font: FONT,
          size: 22,
        })
      );
    } else if (match[3]) {
      // Code
      runs.push(
        new TextRun({
          text: match[4],
          font: "Consolas",
          size: 20,
          shading: { type: ShadingType.CLEAR, fill: "E8E8E8", color: "auto" },
        })
      );
    } else if (match[5]) {
      // Link [text](url)
      runs.push(
        new TextRun({
          text: match[6],
          font: FONT,
          size: 22,
          color: BLUE,
          underline: { type: UnderlineType.SINGLE },
        })
      );
    } else if (match[8]) {
      // Wikilink [[text]]
      runs.push(
        new TextRun({
          text: match[9],
          font: FONT,
          size: 22,
          color: BLUE,
          underline: { type: UnderlineType.SINGLE },
        })
      );
    }

    lastIndex = match.index + match[0].length;
  }

  // Add remaining plain text
  if (lastIndex < text.length) {
    runs.push(
      new TextRun({
        text: text.substring(lastIndex),
        font: FONT,
        size: 22,
      })
    );
  }

  if (runs.length === 0) {
    runs.push(new TextRun({ text, font: FONT, size: 22 }));
  }

  return runs;
}

// Build table from parsed data
function buildTable(rows) {
  const numCols = rows[0].length;
  const totalWidthDxa = 9360; // ~6.5 inches in DXA (twips/20)... actually let's use twips
  const totalTwips = convertInchesToTwip(6.5);
  const colWidth = Math.floor(totalTwips / numCols);
  const columnWidths = Array(numCols).fill(colWidth);

  const borderStyle = {
    style: BorderStyle.SINGLE,
    size: 1,
    color: "BFBFBF",
  };
  const borders = {
    top: borderStyle,
    bottom: borderStyle,
    left: borderStyle,
    right: borderStyle,
  };

  const tableRows = rows.map((row, rowIdx) => {
    const cells = row.map((cellText, colIdx) => {
      const isHeader = rowIdx === 0;
      const inlineRuns = parseInline(cellText);
      // Override font color for header
      const cellRuns = isHeader
        ? inlineRuns.map(
            (r) =>
              new TextRun({
                text: r.root && r.root[1] ? undefined : undefined,
                font: FONT,
                size: 22,
                bold: true,
                color: WHITE,
                children: undefined,
              })
          )
        : inlineRuns;

      // For header, just create simple bold white text
      const para = isHeader
        ? new Paragraph({
            children: [
              new TextRun({
                text: cellText.replace(/\*\*/g, ""),
                font: FONT,
                size: 22,
                bold: true,
                color: WHITE,
              }),
            ],
            spacing: { before: 40, after: 40 },
          })
        : new Paragraph({
            children: inlineRuns,
            spacing: { before: 40, after: 40 },
          });

      return new TableCell({
        children: [para],
        width: { size: colWidth, type: WidthType.DXA },
        borders,
        shading: isHeader
          ? { type: ShadingType.CLEAR, fill: BLUE, color: "auto" }
          : rowIdx % 2 === 0
          ? { type: ShadingType.CLEAR, fill: LIGHT_GRAY, color: "auto" }
          : undefined,
        margins: {
          top: convertInchesToTwip(0.04),
          bottom: convertInchesToTwip(0.04),
          left: convertInchesToTwip(0.08),
          right: convertInchesToTwip(0.08),
        },
      });
    });

    return new TableRow({ children: cells });
  });

  return new Table({
    rows: tableRows,
    width: { size: totalTwips, type: WidthType.DXA },
    columnWidths,
  });
}

// Build the document
const blocks = parseMarkdown(mdContent);
const children = [];

// --- Title Page ---
children.push(
  new Paragraph({ spacing: { before: 3000 } }),
  new Paragraph({
    children: [
      new TextRun({
        text: "Constellation",
        font: FONT,
        size: 72,
        bold: true,
        color: BLUE,
      }),
    ],
    alignment: AlignmentType.CENTER,
    spacing: { after: 200 },
  }),
  new Paragraph({
    children: [
      new TextRun({
        text: "Personal Knowledge Management",
        font: FONT,
        size: 32,
        color: "666666",
      }),
    ],
    alignment: AlignmentType.CENTER,
    spacing: { after: 600 },
  }),
  new Paragraph({
    children: [
      new TextRun({
        text: "\u2500".repeat(40),
        font: FONT,
        size: 24,
        color: BLUE,
      }),
    ],
    alignment: AlignmentType.CENTER,
    spacing: { after: 600 },
  }),
  new Paragraph({
    children: [
      new TextRun({
        text: "User Manual",
        font: FONT,
        size: 52,
        bold: true,
        color: DARK_BLUE,
      }),
    ],
    alignment: AlignmentType.CENTER,
    spacing: { after: 400 },
  }),
  new Paragraph({
    children: [
      new TextRun({
        text: "Version 0.3.4",
        font: FONT,
        size: 28,
        color: "666666",
      }),
    ],
    alignment: AlignmentType.CENTER,
    spacing: { after: 200 },
  }),
  new Paragraph({
    children: [
      new TextRun({
        text: "March 2026",
        font: FONT,
        size: 28,
        color: "666666",
      }),
    ],
    alignment: AlignmentType.CENTER,
    spacing: { after: 200 },
  }),
  new Paragraph({
    children: [
      new TextRun({
        text: "uconstellation.world",
        font: FONT,
        size: 24,
        color: BLUE,
      }),
    ],
    alignment: AlignmentType.CENTER,
    spacing: { after: 200 },
  }),
  new Paragraph({
    children: [new PageBreak()],
  })
);

// --- Table of Contents ---
children.push(
  new Paragraph({
    children: [
      new TextRun({
        text: "Table of Contents",
        font: FONT,
        size: 52,
        bold: true,
        color: DARK_BLUE,
      }),
    ],
    spacing: { after: 400 },
  }),
  new TableOfContents("Table of Contents", {
    hyperlink: true,
    headingStyleRange: "1-3",
  }),
  new Paragraph({
    children: [new PageBreak()],
  })
);

// --- Main Content ---
let skipFirstH1 = true;

for (const block of blocks) {
  switch (block.type) {
    case "h1":
      if (skipFirstH1) {
        skipFirstH1 = false;
        continue; // Skip the title, we have a title page
      }
      children.push(
        new Paragraph({
          children: [
            new TextRun({
              text: block.content,
              font: FONT,
              size: 64,
              bold: true,
              color: DARK_BLUE,
            }),
          ],
          heading: HeadingLevel.HEADING_1,
          spacing: { before: 480, after: 240 },
        })
      );
      break;

    case "h2":
      children.push(
        new Paragraph({
          children: [
            new TextRun({
              text: block.content,
              font: FONT,
              size: 52,
              bold: true,
              color: BLUE,
            }),
          ],
          heading: HeadingLevel.HEADING_2,
          spacing: { before: 360, after: 200 },
        })
      );
      break;

    case "h3":
      children.push(
        new Paragraph({
          children: [
            new TextRun({
              text: block.content,
              font: FONT,
              size: 44,
              bold: true,
              color: DARK_BLUE,
            }),
          ],
          heading: HeadingLevel.HEADING_3,
          spacing: { before: 280, after: 160 },
        })
      );
      break;

    case "h4":
      children.push(
        new Paragraph({
          children: [
            new TextRun({
              text: block.content,
              font: FONT,
              size: 28,
              bold: true,
              color: DARK_BLUE,
            }),
          ],
          heading: HeadingLevel.HEADING_4,
          spacing: { before: 240, after: 120 },
        })
      );
      break;

    case "paragraph": {
      // Check for version/footer line at the end
      const stripped = block.content.replace(/^\*/, "").replace(/\*$/, "");
      if (
        stripped.startsWith("Constellation User Manual") ||
        stripped.startsWith("uconstellation.world")
      ) {
        continue; // Skip, we have footer
      }

      // Tip/Note callout detection
      if (
        block.content.startsWith("> **Tip") ||
        block.content.startsWith("> **Note")
      ) {
        children.push(
          new Paragraph({
            children: parseInline(block.content.replace(/^>\s*/, "")),
            border: {
              left: { style: BorderStyle.SINGLE, size: 6, color: BLUE },
            },
            indent: { left: convertInchesToTwip(0.3) },
            shading: { type: ShadingType.CLEAR, fill: "EBF5FB", color: "auto" },
            spacing: { before: 120, after: 120 },
          })
        );
        break;
      }

      children.push(
        new Paragraph({
          children: parseInline(block.content),
          spacing: { before: 80, after: 80 },
        })
      );
      break;
    }

    case "ul":
      for (const item of block.items) {
        children.push(
          new Paragraph({
            children: parseInline(item),
            numbering: { reference: "bullet-list", level: 0 },
            spacing: { before: 40, after: 40 },
          })
        );
      }
      break;

    case "ol":
      for (let idx = 0; idx < block.items.length; idx++) {
        children.push(
          new Paragraph({
            children: parseInline(block.items[idx]),
            numbering: { reference: "numbered-list", level: 0 },
            spacing: { before: 40, after: 40 },
          })
        );
      }
      break;

    case "table":
      children.push(buildTable(block.rows));
      children.push(new Paragraph({ spacing: { after: 120 } }));
      break;

    case "code":
      for (const codeLine of block.content.split("\n")) {
        children.push(
          new Paragraph({
            children: [
              new TextRun({
                text: codeLine || " ",
                font: "Consolas",
                size: 20,
                color: "333333",
              }),
            ],
            shading: { type: ShadingType.CLEAR, fill: "F5F5F5", color: "auto" },
            indent: { left: convertInchesToTwip(0.3) },
            spacing: { before: 0, after: 0 },
          })
        );
      }
      children.push(new Paragraph({ spacing: { after: 120 } }));
      break;
  }
}

const doc = new Document({
  styles: {
    default: {
      document: {
        run: {
          font: FONT,
          size: 22,
        },
      },
      heading1: {
        run: {
          font: FONT,
          size: 64,
          bold: true,
          color: DARK_BLUE,
        },
        paragraph: {
          spacing: { before: 480, after: 240 },
        },
      },
      heading2: {
        run: {
          font: FONT,
          size: 52,
          bold: true,
          color: BLUE,
        },
        paragraph: {
          spacing: { before: 360, after: 200 },
        },
      },
      heading3: {
        run: {
          font: FONT,
          size: 44,
          bold: true,
          color: DARK_BLUE,
        },
        paragraph: {
          spacing: { before: 280, after: 160 },
        },
      },
      heading4: {
        run: {
          font: FONT,
          size: 28,
          bold: true,
          color: DARK_BLUE,
        },
        paragraph: {
          spacing: { before: 240, after: 120 },
        },
      },
    },
  },
  numbering: {
    config: [
      {
        reference: "bullet-list",
        levels: [
          {
            level: 0,
            format: LevelFormat.BULLET,
            text: "\u2022",
            alignment: AlignmentType.LEFT,
            style: {
              paragraph: {
                indent: {
                  left: convertInchesToTwip(0.5),
                  hanging: convertInchesToTwip(0.25),
                },
              },
            },
          },
        ],
      },
      {
        reference: "numbered-list",
        levels: [
          {
            level: 0,
            format: LevelFormat.DECIMAL,
            text: "%1.",
            alignment: AlignmentType.LEFT,
            style: {
              paragraph: {
                indent: {
                  left: convertInchesToTwip(0.5),
                  hanging: convertInchesToTwip(0.25),
                },
              },
            },
          },
        ],
      },
    ],
  },
  sections: [
    {
      properties: {
        page: {
          size: {
            width: convertInchesToTwip(8.5),
            height: convertInchesToTwip(11),
          },
          margin: {
            top: convertInchesToTwip(1),
            bottom: convertInchesToTwip(1),
            left: convertInchesToTwip(1),
            right: convertInchesToTwip(1),
          },
        },
      },
      headers: {
        default: new Header({
          children: [
            new Paragraph({
              children: [
                new TextRun({
                  text: "Constellation PKM",
                  font: FONT,
                  size: 18,
                  color: BLUE,
                  bold: true,
                }),
                new TextRun({
                  text: "  |  User Manual",
                  font: FONT,
                  size: 18,
                  color: "999999",
                }),
              ],
              border: {
                bottom: {
                  style: BorderStyle.SINGLE,
                  size: 1,
                  color: BLUE,
                },
              },
              spacing: { after: 200 },
            }),
          ],
        }),
      },
      footers: {
        default: new Footer({
          children: [
            new Paragraph({
              children: [
                new TextRun({
                  text: "uconstellation.world",
                  font: FONT,
                  size: 16,
                  color: "999999",
                }),
                new TextRun({
                  text: "\t\t",
                }),
                new TextRun({
                  text: "Page ",
                  font: FONT,
                  size: 16,
                  color: "999999",
                }),
                new TextRun({
                  children: [PageNumber.CURRENT],
                  font: FONT,
                  size: 16,
                  color: "999999",
                }),
              ],
              border: {
                top: {
                  style: BorderStyle.SINGLE,
                  size: 1,
                  color: "CCCCCC",
                },
              },
              tabStops: [
                {
                  type: TabStopType.RIGHT,
                  position: convertInchesToTwip(6.5),
                },
              ],
            }),
          ],
        }),
      },
      children,
    },
  ],
});

Packer.toBuffer(doc).then((buffer) => {
  fs.writeFileSync(
    "E:\\مشاريع كلاود\\Constellation\\docs\\Constellation User Manual (AR).docx",
    buffer
  );
  console.log("AR DOCX generated successfully!");
});
