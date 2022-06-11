use super::Element;
use super::ElementDisplay;
use super::Token;
use rctree::Node;

/// Struct for parsing BbCode Tokens into an Element tree.
pub struct Parser {
    /// DOM root
    root: Node<Element>,
    /// Current traversal node.
    node: Node<Element>,
}

impl Parser {
    pub fn new() -> Self {
        // The rctree's Node<> is a modified RefCell, so cloning is just a ref.
        // See: https://docs.rs/rctree/latest/rctree/struct.Node.html#impl-Clone
        let root = Node::new(Element::new_root());
        let node = root.clone();

        Self { root, node }
    }

    pub fn parse(&mut self, tokens: &[Token]) -> Node<Element> {
        for token in tokens {
            match token {
                Token::Null => {
                    log::warn!("BbCode Lexer received Null instruction, which should not happen.");
                }
                Token::Linebreak => self.add_linebreak(token),
                Token::Tag(_, _) => self.open_tag(token, Element::new_from_token(token)),
                Token::TagClose(tag) => self.close_tag(token, tag),
                Token::Text(text) => self.add_text(text),
            }
        }

        // Cleanly close all open tags.
        while let Some(_) = self.node.parent() {
            self.close_open_tag();
        }
        self.insert_contents_as_node();

        // Unbind and return
        let ast = self.root.clone();
        self.root = Node::new(Element::new_root());
        self.node = self.root.clone();
        ast
    }

    fn add_linebreak(&mut self, token: &Token) {
        if !self.node.borrow().is_void() {
            self.insert_element(Element::new_from_token(token));
        } else {
            unreachable!();
        }
    }

    fn add_text(&mut self, text: &String) {
        self.node.borrow_mut().add_text(text);
    }

    // Attempts to close the currently open tag.
    fn close_open_tag(&mut self) {
        match self.node.parent() {
            Some(parent) => {
                self.insert_contents_as_node();
                self.node = parent;
            }
            None => unreachable!(),
        };
    }

    /// Attempts to close tag, or all tags to tag we are closing.
    fn close_tag(&mut self, token: &Token, tag: &String) {
        let mut tag_matched = false;
        let mut closed_tags = 0;

        if tag.len() < 1 {
            log::warn!("Attempted to close a tag with no name.");
        }

        let mut cursor = Some(self.node.clone());
        while let Some(node) = cursor {
            {
                let el = node.borrow();

                // Check if this element is the same tag as what we're closing.
                tag_matched = el.is_tag(tag);

                // Handle nested closure depending on what this element is.
                if match el.get_display_type() {
                    // Inline tags may be closed by early termination of other tags.
                    ElementDisplay::Inline => true,
                    // Block tags may never be closed by other tags.
                    ElementDisplay::Block => tag_matched,
                } {
                    // Increment counter so we know how many parents we are moving up.
                    closed_tags += 1;
                }
                // Break if we can't close this.
                else {
                    break;
                }
            }

            // If we matched, we end the search now.
            if tag_matched {
                break;
            }

            // If we did not match, we can continue the search.
            cursor = node.parent();
        }

        // If we did not find the tag, we add the closing tag as text instead.
        if !tag_matched {
            return self.add_text(&token.to_tag_string());
        }

        // Close all tags needed.
        while closed_tags > 0 {
            match self.node.parent() {
                Some(_) => self.close_open_tag(),
                None => unreachable!(),
            };
            closed_tags -= 1;
        }
    }

    fn insert_contents_as_node(&mut self) {
        // rctree will panic if you try DOM manipulation with borrowed elements.
        let el = {
            let mut mutel = self.node.borrow_mut();
            mutel.extract_contents()
        };

        // Append text element, if it was created.
        if let Some(el) = el {
            self.node.append(Node::new(el));
        }
    }

    fn insert_element(&mut self, el: Element) -> Node<Element> {
        self.insert_contents_as_node();

        // Append the linebreak itself, if we can.
        let node = Node::new(el);
        self.node.append(node.clone());
        node
    }

    /// Attempts to add element as child to current node and move current node to new element.
    fn open_tag(&mut self, token: &Token, el: Element) {
        // Pulling from nodes in a lifetime is cleaner.
        let (parentable, literal, void) = {
            let mutel = self.node.borrow();
            (mutel.can_parent(&el), mutel.is_literal(), mutel.is_void())
        };

        if parentable {
            // Insert the element and move our pointer.
            self.node = self.insert_element(el);
            return;
        }
        // Literals consume tags as literal text instead of parsing them.
        else if literal {
            self.add_text(&token.to_tag_string());
            return;
        }
        // Voids should not be accepting tokens.
        else if void {
            log::warn!("Attempting to add token to a void element.");
            return;
        }

        unreachable!()
    }
}

mod tests {
    #[test]
    fn add_text_to_root() {
        use super::{Parser, Token};

        let mut parser = Parser::new();
        parser.parse(&[Token::Text("Foobar".to_owned())]);

        assert_eq!(parser.root, parser.root);
        assert_eq!(
            parser.root.first_child().unwrap().borrow().get_contents(),
            Some(&"Foobar".to_string())
        );
    }

    #[test]
    fn add_bold_to_root() {
        use super::{Parser, Token};

        let mut parser = Parser::new();
        parser.parse(&[
            Token::Tag("b".to_owned(), None),
            Token::Text("Foobar".to_owned()),
            Token::TagClose("b".to_owned()),
        ]);

        assert_eq!(parser.root.borrow().get_contents(), None);
        match parser.node.first_child() {
            Some(child) => {
                assert_eq!(child.borrow().get_tag_name(), Some(&"b".to_string()));
                match child.first_child() {
                    Some(child) => {
                        assert_eq!(child.borrow().get_contents(), Some(&"Foobar".to_string()));
                    }
                    None => unreachable!(),
                }
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn add_em_to_bold_and_early_terminate() {
        use super::{Parser, Token};

        let mut parser = Parser::new();
        parser.parse(&[
            Token::Tag("b".to_owned(), None),
            Token::Tag("i".to_owned(), None),
            Token::Text("Foobar".to_owned()),
            Token::TagClose("b".to_owned()),
        ]);

        assert_eq!(parser.root.borrow().get_contents(), None);
        assert_eq!(parser.node, parser.root);
    }

    #[test]
    fn root_linebreak() {
        use super::{Parser, Token};

        let mut parser = Parser::new();
        let ast = parser.parse(&[
            Token::Text("a".to_owned()),
            Token::Linebreak,
            Token::Text("b".to_owned()),
        ]);

        let children = ast.children();
        assert_eq!(children.count(), 3);

        let mut children = ast.children();
        assert_eq!(
            children.nth(0).unwrap().borrow().get_contents(),
            Some(&"a".to_owned())
        );

        let mut children = ast.children();
        assert_eq!(children.nth(1).unwrap().borrow().is_void(), true);

        let mut children = ast.children();
        assert_eq!(
            children.nth(2).unwrap().borrow().get_contents(),
            Some(&"b".to_owned())
        );
    }

    #[test]
    fn root_wont_close() {
        use super::{Parser, Token};

        let mut parser = Parser::new();
        parser.parse(&[Token::TagClose("quote".to_owned())]);

        assert_eq!(
            parser.root.first_child().unwrap().borrow().get_contents(),
            Some(&"[/quote]".to_owned())
        );
        assert_eq!(parser.node, parser.root);
    }
}
