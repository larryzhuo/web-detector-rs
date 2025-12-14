#![deny(clippy::all)]

use ahash::AHasher;
use napi::Result;
use napi_derive::napi;
use scraper::{ElementRef, Html, Node};
use serde::Serialize;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[napi]
pub fn plus_100(input: u32) -> u32 {
  input + 100
}

/// 获取节点名称
fn get_node_name(node: &Node) -> &str {
  match node {
    Node::Element(e) => e.name.local.as_ref(),
    _ => "",
  }
}

/// 为 DOM 节点生成结构指纹（忽略动态属性值，只关注标签结构）
fn structural_hash(element: &ElementRef) -> u64 {
  let mut hasher = AHasher::default();
  element.value().name().hash(&mut hasher);

  // 子元素数量（粗略但有效）
  let child_count = element
    .children()
    .filter(|n| n.value().is_element())
    .count();
  child_count.hash(&mut hasher);

  // 递归子结构（仅标签名）
  for child in element.children().filter(|n| n.value().is_element()) {
    // 注意：这里我们需要将 NodeRef 转换为 ElementRef 来递归
    if let Some(element_ref) = ElementRef::wrap(child.clone()) {
      structural_hash(&element_ref).hash(&mut hasher);
    }
  }

  // 是否包含常见内容标签
  let has_img = element
    .descendants()
    .any(|n| n.value().is_element() && get_node_name(n.value()) == "img");
  let has_link = element
    .descendants()
    .any(|n| n.value().is_element() && get_node_name(n.value()) == "a");
  has_img.hash(&mut hasher);
  has_link.hash(&mut hasher);

  hasher.finish()
}

/// 生成近似 CSS selector 路径（用于定位）
fn generate_css_selector(element: &ElementRef, _doc: &Html) -> String {
  let mut path = Vec::new();
  let mut current = *element;

  loop {
    let parent_opt = current.parent().and_then(|p| ElementRef::wrap(p));

    if let Some(parent) = parent_opt {
      let tag = current.value().name();

      // 获取当前节点在其父级中的位置（忽略文本/注释节点）
      let siblings: Vec<_> = parent
        .children()
        .filter(|n| n.value().is_element())
        .collect();

      if let Some(pos) = siblings.iter().position(|s| s.id() == current.id()) {
        path.push(format!("{}:nth-child({})", tag, pos + 1));
      } else {
        path.push(tag.to_string());
      }

      current = parent;

      if parent.value().name() == "html" {
        break;
      }
    } else {
      break;
    }
  }

  path.reverse();
  if path.is_empty() {
    current.value().name().to_string()
  } else {
    path.join(" > ")
  }
}

/// 单个列表容器的结果
#[derive(Serialize)]
#[napi(object)]
pub struct ListContainer {
  pub selector: String,
  pub item_count: u32,
  pub score: i32,
}

/// 主函数：检测所有列表容器
#[napi]
pub fn detect_lists(html: String) -> Result<Vec<ListContainer>> {
  let doc = Html::parse_document(&html);
  let root = doc.root_element();

  let mut candidates: Vec<(ElementRef, i32)> = Vec::new();

  // 递归遍历所有可能的容器
  let mut stack: Vec<ElementRef> = root
    .children()
    .filter(|n| n.value().is_element())
    .filter_map(|n| ElementRef::wrap(n.clone()))
    .collect();

  while let Some(node) = stack.pop() {
    let tag = node.value().name();
    // 只考虑常见容器标签
    if matches!(
      tag,
      "div" | "ul" | "ol" | "section" | "article" | "main" | "table"
    ) {
      let children: Vec<ElementRef> = node
        .children()
        .filter(|n| n.value().is_element())
        .filter_map(|n| ElementRef::wrap(n.clone()))
        .collect();

      if children.len() >= 3 {
        // 统计子节点结构重复度
        let mut hash_freq: HashMap<u64, usize> = HashMap::new();
        for child in &children {
          let h: u64 = structural_hash(child);
          *hash_freq.entry(h).or_insert(0) += 1;
        }

        if let Some(&max_freq) = hash_freq.values().max() {
          if max_freq >= 3 {
            // 打分：重复度 + 内容丰富度
            let has_img = children.iter().any(|c| {
              c.descendants()
                .any(|n| n.value().is_element() && get_node_name(n.value()) == "img")
            });
            let has_link = children.iter().any(|c| {
              c.descendants()
                .any(|n| n.value().is_element() && get_node_name(n.value()) == "a")
            });

            let score =
              (max_freq as i32) * 10 + if has_img { 5 } else { 0 } + if has_link { 3 } else { 0 };

            candidates.push((node, score));
          }
        }
      }
    }

    // 继续遍历子节点
    for child in node.children().filter(|n| n.value().is_element()) {
      if let Some(child_element) = ElementRef::wrap(child.clone()) {
        stack.push(child_element);
      }
    }
  }

  // 按分数降序排序
  candidates.sort_by(|a, b| b.1.cmp(&a.1));

  // 生成结果
  let results: Vec<ListContainer> = candidates
    .into_iter()
    .map(|(elem, score)| ListContainer {
      selector: generate_css_selector(&elem, &doc),
      item_count: elem.children().filter(|n| n.value().is_element()).count() as u32,
      score,
    })
    .collect();

  Ok(results)
}
