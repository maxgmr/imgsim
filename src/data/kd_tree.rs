// #![warn(missing_docs)]
//
// use crate::ImgsimImage;
//
// #[derive(Debug)]
// pub struct KDTree {
//     tree: Vec<KDTNode>,
//     space: RGBBound,
//     depth: usize,
// }
// impl KDTree {
//     pub fn build(imgsim_image: &ImgsimImage) -> KDTree {
//         let mut tree = Vec::new();
//         let space = Default::default();
//         let items: Vec<((u32, u32), (u8, u8, u8))> = imgsim_image
//             .rgba_image()
//             .enumerate_pixels()
//             .map(|(x, y, pixel)| {
//                 let image::Rgba([r, g, b, _]) = *pixel;
//                 ((x, y), (r, g, b))
//             })
//             .collect();
//
//         let depth: usize = KDTree::build_tree(&space, items, &mut tree);
//         KDTree { space, tree, depth }
//     }
//
//     fn build_tree(
//         space: &RGBBound,
//         items: Vec<((u32, u32), (u8, u8, u8))>,
//         tree: &mut Vec<KDTNode>,
//     ) -> usize {
//         if items.len() <= 1 {
//             if let Some(((x, y), (r, g, b))) = items.get(0) {
//                 tree.push(KDTNode::Leaf {
//                     pixel: ((*x, *y), (*r, *g, *b)),
//                 });
//                 return 1;
//             } else {
//                 return 0;
//             }
//         }
//
//         let split_plane = space.get_longest_plane();
//         let split_coord = match split_plane {
//             Plane::R => KDTree::get_median(items.iter().map(|(_, (r, _, _))| *r).collect()),
//             Plane::G => KDTree::get_median(items.iter().map(|(_, (_, g, _))| *g).collect()),
//             Plane::B => KDTree::get_median(items.iter().map(|(_, (_, _, b))| *b).collect()),
//         };
//
//         // TODO can definitely make this cleaner
//         let left_split = match split_plane {
//             Plane::R => items
//                 .iter()
//                 .filter(|(_, (r, _, _))| r > &split_coord)
//                 .cloned()
//                 .collect(),
//             Plane::G => items
//                 .iter()
//                 .filter(|(_, (_, g, _))| g > &split_coord)
//                 .cloned()
//                 .collect(),
//             Plane::B => items
//                 .iter()
//                 .filter(|(_, (_, _, b))| b > &split_coord)
//                 .cloned()
//                 .collect(),
//         };
//         let right_split = match split_plane {
//             Plane::R => items
//                 .iter()
//                 .filter(|(_, (r, _, _))| r <= &split_coord)
//                 .cloned()
//                 .collect(),
//             Plane::G => items
//                 .iter()
//                 .filter(|(_, (_, g, _))| g <= &split_coord)
//                 .cloned()
//                 .collect(),
//             Plane::B => items
//                 .iter()
//                 .filter(|(_, (_, _, b))| b <= &split_coord)
//                 .cloned()
//                 .collect(),
//         };
//     }
//
//     fn get_median(mut list: Vec<u8>) -> u8 {
//         list.sort_unstable();
//         if let Some(val) = list.get((list.len() as f32 / 2.0).ceil() as usize) {
//             *val
//         } else {
//             *list
//                 .get((list.len() as f32 / 2.0).floor() as usize)
//                 .unwrap()
//         }
//     }
// }
//
// #[derive(Debug)]
// enum Plane {
//     R,
//     G,
//     B,
// }
//
// #[derive(Debug)]
// pub enum KDTNode {
//     Leaf {
//         pixel: ((u32, u32), (u8, u8, u8)),
//     },
//     Node {
//         space: RGBBound,
//         count: usize,
//         weighted_cent: (u64, u64, u64),
//     },
// }
//
// #[derive(Debug)]
// pub struct RGBBound {
//     min: (u8, u8, u8),
//     max: (u8, u8, u8),
// }
// impl RGBBound {
//     /// Create new RGBBound from two points in sRGB colour space.
//     pub fn new(min: (u8, u8, u8), max: (u8, u8, u8)) -> RGBBound {
//         RGBBound { min, max }
//     }
//
//     /// Create empty RGBBound.
//     pub fn empty() -> RGBBound {
//         RGBBound::new((0, 0, 0), (255, 255, 255))
//     }
//
//     /// Check for intersection with another RGBBound.
//     pub fn intersect(&self, other: &RGBBound) -> bool {
//         ((self.min().0 < other.max().0) && (self.max().0 > other.min().0))
//             && ((self.min().1 < other.max().1) && (self.max().1 > other.min().1))
//             && ((self.min().2 < other.max().2) && (self.max().2 > other.min().2))
//     }
//
//     /// Consume another RGBBound.
//     pub fn merge(&mut self, other: RGBBound) {
//         self.min = (
//             self.min().0.min(other.min().0),
//             self.min().1.min(other.min().1),
//             self.min().2.min(other.min().2),
//         );
//         self.max = (
//             self.max().0.max(other.max().0),
//             self.max().1.max(other.max().1),
//             self.max().2.max(other.max().2),
//         );
//     }
//
//     /// Get min RGB.
//     pub fn min(&self) -> &(u8, u8, u8) {
//         &self.min
//     }
//
//     /// Get max RGB.
//     pub fn max(&self) -> &(u8, u8, u8) {
//         &self.max
//     }
//
//     /// Get length of R axis.
//     pub fn r_len(&self) -> u8 {
//         &self.max.0 - &self.min.0
//     }
//
//     /// Get length of G axis.
//     pub fn g_len(&self) -> u8 {
//         &self.max.1 - &self.min.1
//     }
//
//     /// Get length of B axis.
//     pub fn b_len(&self) -> u8 {
//         &self.max.2 - &self.min.2
//     }
//
//     pub fn get_longest_plane(&self) -> Plane {
//         if self.r_len() > self.b_len() {
//             if self.r_len() > self.g_len() {
//                 return Plane::R;
//             } else {
//                 return Plane::G;
//             }
//         } else if self.b_len() > self.g_len() {
//             return Plane::B;
//         } else {
//             return Plane::G;
//         }
//     }
//
//     pub fn is_in(&self, (r, g, b): &(u8, u8, u8)) -> bool {
//         (r > &self.min().0)
//             && (r < &self.max().0)
//             && (g > &self.min().1)
//             && (g < &self.max().1)
//             && (b > &self.min().2)
//             && (b < &self.max().2)
//     }
// }
// impl Default for RGBBound {
//     fn default() -> RGBBound {
//         RGBBound::empty()
//     }
// }
//
