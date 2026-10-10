//! Resolve parent-relative placement and clipping into leaf references, without pixel buffers.
use crate::Error;
pub const MAX_NODES: usize = 12;
pub const LEAVES: usize = 4;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}
impl Rect {
    pub fn bytes(self) -> Result<usize, Error> {
        self.width
            .checked_mul(self.height)
            .and_then(|n| n.checked_mul(4))
            .ok_or(Error::Geometry)
    }
    fn end(self) -> Result<(usize, usize), Error> {
        Ok((
            self.x.checked_add(self.width).ok_or(Error::Geometry)?,
            self.y.checked_add(self.height).ok_or(Error::Geometry)?,
        ))
    }
    fn intersect(self, other: Self) -> Result<Self, Error> {
        let (right, bottom) = self.end()?;
        let (oright, obottom) = other.end()?;
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        Ok(Self {
            x,
            y,
            width: right.min(oright).saturating_sub(x),
            height: bottom.min(obottom).saturating_sub(y),
        })
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Node {
    pub parent: Option<usize>,
    pub rect: Rect,
    pub source: Option<usize>,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Draw {
    pub target: Rect,
    pub source_x: usize,
    pub source_y: usize,
    pub source_width: usize,
    pub source_height: usize,
}
pub struct Plan {
    pub draws: [Option<Draw>; LEAVES],
    pub width: usize,
    pub height: usize,
    pub visited: usize,
}
pub fn resolve(nodes: &[Node], width: usize, height: usize) -> Result<Plan, Error> {
    if nodes.is_empty() || nodes.len() > MAX_NODES || width == 0 || height == 0 {
        return Err(Error::Geometry);
    }
    Rect {
        width,
        height,
        ..Rect::default()
    }
    .bytes()?;
    let mut origins = [(0usize, 0usize); MAX_NODES];
    let mut clips = [Rect::default(); MAX_NODES];
    let mut plan = Plan {
        draws: [None; LEAVES],
        width,
        height,
        visited: nodes.len(),
    };
    for (index, node) in nodes.iter().enumerate() {
        let (origin, clip) = if index == 0 {
            if node.parent.is_some() || node.source.is_some() {
                return Err(Error::Geometry);
            }
            (
                (0, 0),
                Rect {
                    x: 0,
                    y: 0,
                    width,
                    height,
                },
            )
        } else {
            let parent = node.parent.filter(|&p| p < index).ok_or(Error::Geometry)?;
            if nodes[parent].source.is_some() {
                return Err(Error::Geometry);
            }
            (origins[parent], clips[parent])
        };
        let world = Rect {
            x: origin.0.checked_add(node.rect.x).ok_or(Error::Geometry)?,
            y: origin.1.checked_add(node.rect.y).ok_or(Error::Geometry)?,
            ..node.rect
        };
        world.end()?;
        origins[index] = (world.x, world.y);
        clips[index] = world.intersect(clip)?;
        if let Some(source) = node.source {
            if source >= LEAVES || plan.draws[source].is_some() {
                return Err(Error::Geometry);
            }
            let target = clips[index];
            for draw in plan.draws.iter().flatten() {
                let overlap = target.intersect(draw.target)?;
                if overlap.width != 0 && overlap.height != 0 {
                    return Err(Error::Overlap);
                }
            }
            plan.draws[source] = Some(Draw {
                target,
                source_x: target.x - world.x,
                source_y: target.y - world.y,
                source_width: world.width,
                source_height: world.height,
            });
        }
    }
    Ok(plan)
}
/// Four quadrants under a chain of full-frame containers. Depth counts the
/// ancestor buffers a naive implementation would materialize, including root.
pub fn fixture(
    width: usize,
    height: usize,
    depth: usize,
) -> Result<([Node; MAX_NODES], usize), Error> {
    if !(1..=MAX_NODES - LEAVES).contains(&depth) || width < 2 || height < 2 {
        return Err(Error::Geometry);
    }
    let mut nodes = [Node::default(); MAX_NODES];
    for (index, node) in nodes[..depth].iter_mut().enumerate() {
        *node = Node {
            parent: index.checked_sub(1),
            rect: Rect {
                width,
                height,
                ..Rect::default()
            },
            source: None,
        };
    }
    let xs = [(0, width / 2), (width / 2, width - width / 2)];
    let ys = [(0, height / 2), (height / 2, height - height / 2)];
    for (source, node) in nodes[depth..depth + LEAVES].iter_mut().enumerate() {
        let (x, w) = xs[source % 2];
        let (y, h) = ys[source / 2];
        *node = Node {
            parent: Some(depth - 1),
            rect: Rect {
                x,
                y,
                width: w,
                height: h,
            },
            source: Some(source),
        };
    }
    Ok((nodes, depth + LEAVES))
}
