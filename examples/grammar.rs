#![allow(dead_code)]

extern crate nel;

use std::usize;

use nel::tokenize::*;

fn main() {
   let mut builder = Builder::new();

   for f in &[
      expression,
      nary_right,
      nary_operator,
      single,
      parens,
      identifier,
      number,
      number_with_integer,
      number_fractional_only,
      space,
   ] {
      f(&mut builder);
   }

   let (nodes, elements) = builder.destructure();

   nodes
      .iter()
      .enumerate()
      .for_each(|(i, node)| println!("[{:03}] {:?}", i, node));

   println!("---");

   elements
      .iter()
      .enumerate()
      .for_each(|(i, pos)| println!("[{:03}] {}", i, pos));

   println!("---");

   let source = "icons";

   let chars: Vec<_> = source.chars().collect();

   let (toks, _) = tokenize(&chars);

   toks
      .iter()
      .enumerate()
      .for_each(|(i, tok)| println!("[{:03}] {:?}", i, tok));

   println!("================");

   parse_toks(&nodes, &elements, &toks);
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn expression(b: &mut Builder) {
   b.element(Element::NaryRight)
      .sequence()
         .reference(Element::Single)
//         .zero_or_more()
         .zero_or_one()
            .reference(Element::NaryRight)
         .end()
      .end();
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn nary_right(b: &mut Builder) {
   b.element(Element::NaryRight)
      .sequence()
         .reference(Element::Space)
         .reference(Element::NaryOperator)
         .reference(Element::Space)
         .reference(Element::Single)
      .end();
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn nary_operator(b: &mut Builder) {
   b.element(Element::NaryOperator)
      .choice()
         .tok(Tok::Plus)
         .tok(Tok::Minus)
         .tok(Tok::Asterisk)
         .tok(Tok::Slash)
      .end();
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn single(b: &mut Builder) {
   b.element(Element::Single)
      .choice()
         .reference(Element::Identifier)
         .reference(Element::Number)
         .reference(Element::Parens)
      .end();
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn parens(b: &mut Builder) {
   b.element(Element::Parens)
      .sequence()
         .tok(Tok::ParenLeft)
         .reference(Element::Space)
         .reference(Element::Expression)
         .reference(Element::Space)
         .tok(Tok::ParenRight)
      .end();
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn identifier(b: &mut Builder) {
   b.element(Element::Identifier)
      .sequence()
         .tok(Tok::Identifier)
      .end();
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn number(b: &mut Builder) {
   b.element(Element::Number)
      .choice()
         .reference(Element::NumberWithInteger)
         .reference(Element::NumberFractionalOnly)
      .end();
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn number_with_integer(b: &mut Builder) {
   b.element(Element::NumberWithInteger)
      .sequence()
         .tok(Tok::Digits)
         .zero_or_one()
            .tok(Tok::FullStop)
            .zero_or_one()
               .tok(Tok::Digits)
            .end()
         .end()
      .end();
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn number_fractional_only(b: &mut Builder) {
   b.element(Element::NumberFractionalOnly)
      .sequence()
         .tok(Tok::FullStop)
         .tok(Tok::Digits)
      .end();
}

#[cfg_attr(rustfmt, rustfmt_skip)]
fn space(b: &mut Builder) {
   b.element(Element::Space)
      .zero_or_one()
         .tok(Tok::Space)
      .end();
}

#[derive(Debug, Clone, Copy)]
enum Element {
   Expression = 0,
   NaryRight,
   NaryOperator,
   Single,
   Parens,
   Identifier,
   Number,
   NumberWithInteger,
   NumberFractionalOnly,
   Space,
}

const ELS: usize = Element::Space as usize + 1;

#[derive(Debug)]
enum Node {
   Element(Element),
   Reference(Element),
   Tok(Tok),
   Sequence(usize),
   Choice(usize),
   ZeroOrOne(usize),
}

struct Builder {
   nodes: Vec<Node>,
   starts: Vec<usize>,
   elements: [usize; ELS],
}

impl Builder {
   fn new() -> Self {
      Builder {
         nodes: Vec::new(),
         starts: Vec::new(),
         elements: [0; ELS],
      }
   }

   fn destructure(self) -> (Vec<Node>, [usize; ELS]) {
      let Self {
         nodes, elements, ..
      } = self;
      (nodes, elements)
   }

   fn element(&mut self, element: Element) -> &mut Self {
      self.elements[element as usize] = self.nodes.len();
      self.nodes.push(Node::Element(element));
      self
   }

   fn reference(&mut self, element: Element) -> &mut Self {
      self.nodes.push(Node::Reference(element));
      self
   }

   fn tok(&mut self, tok: Tok) -> &mut Self {
      self.nodes.push(Node::Tok(tok));
      self
   }

   fn sequence(&mut self) -> &mut Self {
      self.start(Node::Sequence(0));
      self
   }

   fn choice(&mut self) -> &mut Self {
      self.start(Node::Choice(0));
      self
   }

   fn zero_or_one(&mut self) -> &mut Self {
      self.start(Node::ZeroOrOne(0));
      self
   }

   fn start(&mut self, parent: Node) -> &mut Self {
      self.starts.push(self.nodes.len());
      self.nodes.push(parent);
      self
   }

   fn end(&mut self) -> &mut Self {
      debug_assert!(!self.starts.is_empty());

      let start = self.starts.pop().unwrap();

      debug_assert!(self.nodes.len() > start);

      let end = self.nodes.len();

      match *unsafe { self.nodes.get_unchecked_mut(start) } {
         Node::Sequence(ref mut i) | Node::Choice(ref mut i) | Node::ZeroOrOne(ref mut i) => {
            *i = end
         }
         _ => unreachable!(),
      }

      self
   }
}

#[derive(Debug)]
struct State {
   list_type: ListType,
   current: usize,
   end: usize,
   tok_pos: usize,
}

#[derive(Debug, PartialEq)]
enum ListType {
   Element,
   Sequence,
   Choice,
   ZeroOrOne,
}

impl State {
   fn new(list_type: ListType, current: usize, end: usize, tok_pos: usize) -> Self {
      State {
         list_type,
         current,
         end,
         tok_pos,
      }
   }
}

fn parse_toks(nodes: &[Node], elements: &[usize; ELS], toks: &[Tok]) {
   let mut states: Vec<State> = Vec::new();

   let mut next_pos = elements[Element::Expression as usize];
   let mut tok_pos = 0;

   loop {
      println!("[{:03}] {:?}", next_pos, nodes[next_pos]);

      next_pos = match nodes[next_pos] {
         Node::Element(ref _element) => {
            states.push(State::new(ListType::Element, next_pos, usize::MAX, tok_pos));
            next_pos + 1
         }
         Node::Sequence(end) => {
            states.push(State::new(ListType::Sequence, next_pos, end, tok_pos));
            next_pos + 1
         }
         Node::Choice(end) => {
            states.push(State::new(ListType::Choice, next_pos, end, tok_pos));
            next_pos + 1
         }
         Node::ZeroOrOne(end) => {
            states.push(State::new(ListType::ZeroOrOne, next_pos, end, tok_pos));
            next_pos + 1
         }
         Node::Reference(ref element) => elements[*element as usize],
         Node::Tok(ref tok) => {
            let mut matched = if let Some(tok_src) = toks.get(tok_pos) {
               tok == tok_src
            } else {
               false
            };

            println!("T [{:03}] {:?} M={}", tok_pos, tok, matched);

            tok_pos += 1;
            next_pos += 1;

            loop {
               if let Some(state) = states.last() {
                  println!("<==== {:?}", state);

                  match state.list_type {
                     ListType::ZeroOrOne | ListType::Sequence => if !matched {
                        next_pos = state.end;
                        println!("end");
                     },
                     ListType::Choice => {
                        if matched {
                           next_pos = state.end;
                           println!("end");
                        } else {
                           tok_pos = state.tok_pos;
                        }
                     }
                     ListType::Element => unreachable!(),
                  }

                  if next_pos != state.end {
                     println!("BREAK {} {}", next_pos, state.end);
                     break;
                  }

                  if state.list_type == ListType::ZeroOrOne && !matched {
                     matched = true;
                  }
               } else {
                  unreachable!();
               }

               let el = states.pop();
               println!("POP {:?}", el);

               let pop_again = if let Some(state) = states.last() {
                  state.list_type == ListType::Element
               } else {
                  false
               };

               if pop_again {
                  let el = states.pop();
                  println!("POP {:?}", el);
               }

               println!("M {}", matched);

               if let Some(state) = states.last_mut() {
                  next_pos = state.current;
               } else {
                  return;
               }
            }

            next_pos
         }
      };

      if let Some(state) = states.last_mut() {
         state.current = next_pos;
         println!("====> {:?}", state);
      } else {
         unreachable!();
      }
   }
}

/*

===---===---===---===---===


number = |
   number_with_integer
   number_fractional_only

number_with_integer = (
   one(^digits)
   zero_or_one(
      ^full_stop
      zero_or_one(^digits)
   )
)

number_fractional_only = one(^full_stop) one(^digits)

space = zero_or_one(^space)

identifier = one(^identifier)

single = |
   identifier
   number
   parens

parens = ~
   one(^paren_left)
   space
   expression
   space
   one(^paren_right)

expression = ~
   single
   zero_or_more(nary_right)

nary_right = ~
   space
   one(operators)
   space
   single

operators = |
   ^plus
   ^minus
   ^asterisk
   ^slash


TK = ^digits  | ^full_stop  |
   .          .             .
   0          1


OP = number  |          |
   .         .          .
   0         1


SQ = or-sq-1 | li-sq-2 | 1-tk-1 | 01-sq-4  | 1-tk-2  | 01-tk-1
   .         .         .        .          .         .
   0         1         2        3          4
   ^number
   :
   :
   |         |          |         |
   .         .          .         .
   10        11         12        13


*/