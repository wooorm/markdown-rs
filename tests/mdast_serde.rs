use markdown::mdast::Node;

#[cfg_attr(feature = "serde", test)]
fn test_serde() {
    let markdown = "This is a **test**\n\n> quote content\n> continue";
    let tree = markdown::to_mdast(&markdown, &markdown::ParseOptions::default()).unwrap();
    let json = serde_json::to_string_pretty(&tree).unwrap();
    assert_eq!(
        json,
        r#"{
  "type": "root",
  "children": [
    {
      "type": "paragraph",
      "children": [
        {
          "type": "text",
          "value": "This is a ",
          "position": {
            "start": {
              "line": 1,
              "column": 1,
              "offset": 0
            },
            "end": {
              "line": 1,
              "column": 11,
              "offset": 10
            }
          }
        },
        {
          "type": "strong",
          "children": [
            {
              "type": "text",
              "value": "test",
              "position": {
                "start": {
                  "line": 1,
                  "column": 13,
                  "offset": 12
                },
                "end": {
                  "line": 1,
                  "column": 17,
                  "offset": 16
                }
              }
            }
          ],
          "position": {
            "start": {
              "line": 1,
              "column": 11,
              "offset": 10
            },
            "end": {
              "line": 1,
              "column": 19,
              "offset": 18
            }
          }
        }
      ],
      "position": {
        "start": {
          "line": 1,
          "column": 1,
          "offset": 0
        },
        "end": {
          "line": 1,
          "column": 19,
          "offset": 18
        }
      }
    },
    {
      "type": "blockquote",
      "children": [
        {
          "type": "paragraph",
          "children": [
            {
              "type": "text",
              "value": "quote content\ncontinue",
              "position": {
                "start": {
                  "line": 3,
                  "column": 3,
                  "offset": 22
                },
                "end": {
                  "line": 4,
                  "column": 11,
                  "offset": 46
                }
              }
            }
          ],
          "position": {
            "start": {
              "line": 3,
              "column": 3,
              "offset": 22
            },
            "end": {
              "line": 4,
              "column": 11,
              "offset": 46
            }
          }
        }
      ],
      "position": {
        "start": {
          "line": 3,
          "column": 1,
          "offset": 20
        },
        "end": {
          "line": 4,
          "column": 11,
          "offset": 46
        }
      }
    }
  ],
  "position": {
    "start": {
      "line": 1,
      "column": 1,
      "offset": 0
    },
    "end": {
      "line": 4,
      "column": 11,
      "offset": 46
    }
  }
}"#
    );
    let tree2: Node = serde_json::from_str(&json).unwrap();
    assert!(tree == tree2);
}
