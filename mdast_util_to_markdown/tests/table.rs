use markdown::mdast::{
    AlignKind, Emphasis, InlineCode, Link, Node, Strong, Table, TableCell, TableRow, Text,
};
use mdast_util_to_markdown::to_markdown as to;
use pretty_assertions::assert_eq;

#[test]
fn table_basic() {
    // Simple 2x2 table
    assert_eq!(
        to(&Node::Table(Table {
            align: vec![AlignKind::None, AlignKind::None],
            children: vec![
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "a".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "b".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "c".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "d".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
            ],
            position: None,
        }))
        .unwrap(),
        "| a | b |\n| --- | --- |\n| c | d |\n",
        "should support a simple 2x2 table"
    );

    // Table with header only
    assert_eq!(
        to(&Node::Table(Table {
            align: vec![AlignKind::None],
            children: vec![Node::TableRow(TableRow {
                children: vec![Node::TableCell(TableCell {
                    children: vec![Node::Text(Text {
                        value: "header".to_string(),
                        position: None,
                    })],
                    position: None,
                })],
                position: None,
            })],
            position: None,
        }))
        .unwrap(),
        "| header |\n| ------ |\n",
        "should support a table with header only"
    );

    // Empty table
    assert_eq!(
        to(&Node::Table(Table {
            align: vec![],
            children: vec![],
            position: None,
        }))
        .unwrap(),
        "",
        "should handle empty table"
    );
}

#[test]
fn table_alignment() {
    assert_eq!(
        to(&Node::Table(Table {
            align: vec![AlignKind::Left, AlignKind::Center, AlignKind::Right],
            children: vec![
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "left".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "center".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "right".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "a".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "b".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "c".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
            ],
            position: None,
        }))
        .unwrap(),
        "| left | center | right |\n| :--- | :----: | ----: |\n| a | b | c |\n",
        "should support different alignments"
    );
}

#[test]
fn table_column_width() {
    // Uneven column widths
    assert_eq!(
        to(&Node::Table(Table {
            align: vec![AlignKind::None, AlignKind::None],
            children: vec![
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "short".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "much longer content".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "a".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "b".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
            ],
            position: None,
        }))
        .unwrap(),
        "| short | much longer content |\n| ----- | ------------------- |\n| a | b |\n",
        "should handle uneven column widths"
    );

    // Unicode characters
    assert_eq!(
        to(&Node::Table(Table {
            align: vec![AlignKind::None, AlignKind::None],
            children: vec![
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "中文".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "🎉".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "test".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "ok".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
            ],
            position: None,
        }))
        .unwrap(),
        "| 中文 | 🎉 |\n| ---- | --- |\n| test | ok |\n",
        "should handle Unicode characters"
    );
}

#[test]
fn table_escaping() {
    // Pipes in text
    assert_eq!(
        to(&Node::Table(Table {
            align: vec![AlignKind::None],
            children: vec![Node::TableRow(TableRow {
                children: vec![Node::TableCell(TableCell {
                    children: vec![Node::Text(Text {
                        value: "a | b".to_string(),
                        position: None,
                    })],
                    position: None,
                })],
                position: None,
            }),],
            position: None,
        }))
        .unwrap(),
        "| a \\| b |\n| ----- |\n",
        "should escape pipes in text"
    );

    // Pipes in code spans should NOT be escaped
    assert_eq!(
        to(&Node::Table(Table {
            align: vec![AlignKind::None],
            children: vec![Node::TableRow(TableRow {
                children: vec![Node::TableCell(TableCell {
                    children: vec![Node::InlineCode(InlineCode {
                        value: "a | b".to_string(),
                        position: None,
                    })],
                    position: None,
                })],
                position: None,
            }),],
            position: None,
        }))
        .unwrap(),
        "| `a | b` |\n| ------- |\n",
        "should NOT escape pipes in code spans"
    );

    // Mixed content with pipes
    assert_eq!(
        to(&Node::Table(Table {
            align: vec![AlignKind::None],
            children: vec![Node::TableRow(TableRow {
                children: vec![Node::TableCell(TableCell {
                    children: vec![
                        Node::Text(Text {
                            value: "text | with".to_string(),
                            position: None,
                        }),
                        Node::InlineCode(InlineCode {
                            value: "code | here".to_string(),
                            position: None,
                        }),
                        Node::Text(Text {
                            value: "| more".to_string(),
                            position: None,
                        }),
                    ],
                    position: None,
                })],
                position: None,
            }),],
            position: None,
        }))
        .unwrap(),
        "| text \\| with`code | here`\\| more |\n| ------------------------------ |\n",
        "should handle mixed escaping correctly"
    );
}

#[test]
fn table_inline_elements() {
    // Table with emphasis, strong, and links
    assert_eq!(
        to(&Node::Table(Table {
            align: vec![AlignKind::None, AlignKind::None],
            children: vec![
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![Node::Emphasis(Emphasis {
                                children: vec![Node::Text(Text {
                                    value: "italic".to_string(),
                                    position: None,
                                })],
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Strong(Strong {
                                children: vec![Node::Text(Text {
                                    value: "bold".to_string(),
                                    position: None,
                                })],
                                position: None,
                            })],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![Node::Link(Link {
                                url: "https://example.com".to_string(),
                                title: None,
                                children: vec![Node::Text(Text {
                                    value: "link".to_string(),
                                    position: None,
                                })],
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::InlineCode(InlineCode {
                                value: "code".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
            ],
            position: None,
        }))
        .unwrap(),
        "| *italic* | **bold** |\n| --------------------------- | -------- |\n| [link](https://example.com) | `code` |\n",
        "should support inline elements in cells"
    );
}

#[test]
fn table_edge_cases() {
    // Single column table
    assert_eq!(
        to(&Node::Table(Table {
            align: vec![AlignKind::None],
            children: vec![
                Node::TableRow(TableRow {
                    children: vec![Node::TableCell(TableCell {
                        children: vec![Node::Text(Text {
                            value: "single".to_string(),
                            position: None,
                        })],
                        position: None,
                    })],
                    position: None,
                }),
                Node::TableRow(TableRow {
                    children: vec![Node::TableCell(TableCell {
                        children: vec![Node::Text(Text {
                            value: "column".to_string(),
                            position: None,
                        })],
                        position: None,
                    })],
                    position: None,
                }),
            ],
            position: None,
        }))
        .unwrap(),
        "| single |\n| ------ |\n| column |\n",
        "should support single column tables"
    );

    // Uneven cells per row (should pad missing cells)
    assert_eq!(
        to(&Node::Table(Table {
            align: vec![AlignKind::None, AlignKind::None, AlignKind::None],
            children: vec![
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "a".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "b".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "c".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "d".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "e".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
            ],
            position: None,
        }))
        .unwrap(),
        "| a | b | c |\n| --- | --- | --- |\n| d | e |  |\n",
        "should handle uneven cells per row"
    );

    // Empty cells
    assert_eq!(
        to(&Node::Table(Table {
            align: vec![AlignKind::None, AlignKind::None],
            children: vec![
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "b".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "c".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
            ],
            position: None,
        }))
        .unwrap(),
        "|  | b |\n| --- | --- |\n| c |  |\n",
        "should handle empty cells"
    );
}

#[test]
fn table_errors() {
    // TableRow cannot be serialized alone
    let result = to(&Node::TableRow(TableRow {
        children: vec![Node::TableCell(TableCell {
            children: vec![Node::Text(Text {
                value: "test".to_string(),
                position: None,
            })],
            position: None,
        })],
        position: None,
    }));
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .reason
        .contains("Cannot serialize `TableRow` outside of `Table`"));

    // TableCell cannot be serialized alone
    let result = to(&Node::TableCell(TableCell {
        children: vec![Node::Text(Text {
            value: "test".to_string(),
            position: None,
        })],
        position: None,
    }));
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .reason
        .contains("Cannot serialize `TableCell` outside of `Table`"));
}

#[test]
fn table_complex() {
    // Complex table with multiple formatting types
    assert_eq!(
        to(&Node::Table(Table {
            align: vec![AlignKind::Left, AlignKind::Center, AlignKind::Right],
            children: vec![
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "Feature".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "Status".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::Text(Text {
                                value: "Notes".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
                Node::TableRow(TableRow {
                    children: vec![
                        Node::TableCell(TableCell {
                            children: vec![
                                Node::Strong(Strong {
                                    children: vec![Node::Text(Text {
                                        value: "Tables".to_string(),
                                        position: None,
                                    })],
                                    position: None,
                                }),
                            ],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![Node::InlineCode(InlineCode {
                                value: "done".to_string(),
                                position: None,
                            })],
                            position: None,
                        }),
                        Node::TableCell(TableCell {
                            children: vec![
                                Node::Text(Text {
                                    value: "With ".to_string(),
                                    position: None,
                                }),
                                Node::Emphasis(Emphasis {
                                    children: vec![Node::Text(Text {
                                        value: "alignment".to_string(),
                                        position: None,
                                    })],
                                    position: None,
                                }),
                            ],
                            position: None,
                        }),
                    ],
                    position: None,
                }),
            ],
            position: None,
        }))
        .unwrap(),
        "| Feature | Status | Notes |\n| :--------- | :----: | ---------------: |\n| **Tables** | `done` | With *alignment* |\n",
        "should handle complex table with multiple formatting types"
    );
}
