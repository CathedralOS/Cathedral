//! Scalar opaque workload shared by native host measurements and actual guest mappings.
use crate::{
    Error,
    rows::Row,
    scene::{LEAVES, Plan, Rect},
};
#[derive(Clone, Copy)]
pub struct Source<'a> {
    pub bytes: &'a [u8],
    pub stride: usize,
}
pub fn color(x: usize, y: usize) -> u32 {
    (((x.wrapping_mul(17) & 255) as u32) << 16)
        | (((y.wrapping_mul(29) & 255) as u32) << 8)
        | ((x ^ y) & 255) as u32
}
pub fn leaf(bytes: &mut [u8], rect: Rect, stride: usize) -> Result<usize, Error> {
    rect.x.checked_add(rect.width).ok_or(Error::Bounds)?;
    rect.y.checked_add(rect.height).ok_or(Error::Bounds)?;
    let row_bytes = rect.width.checked_mul(4).ok_or(Error::Bounds)?;
    if stride < row_bytes
        || stride
            .checked_mul(rect.height)
            .is_none_or(|n| n > bytes.len())
    {
        return Err(Error::Bounds);
    }
    for row in 0..rect.height {
        let mut lease = Row::new(&mut bytes[row * stride..row * stride + row_bytes])?;
        for column in 0..rect.width {
            lease.put(column, color(rect.x + column, rect.y + row))?;
        }
    }
    rect.bytes()
}
/// Checked single-allocation row delegation. abort_row models a renderer which
/// stops before completion; callers must not publish the partially drawn back buffer.
pub fn direct(
    bytes: &mut [u8],
    width: usize,
    height: usize,
    depth: usize,
    abort_row: Option<usize>,
) -> Result<usize, Error> {
    if width == 0 || height == 0 || depth == 0 {
        return Err(Error::Geometry);
    }
    let row_bytes = width.checked_mul(4).ok_or(Error::Bounds)?;
    if row_bytes
        .checked_mul(height)
        .is_none_or(|n| n > bytes.len())
    {
        return Err(Error::Bounds);
    }
    for row in 0..height {
        if abort_row == Some(row) {
            return Err(Error::Incomplete);
        }
        let mut lease = Row::new(&mut bytes[row * row_bytes..(row + 1) * row_bytes])?;
        for _ in 1..depth {
            lease = lease.narrow(0, width)?;
        }
        let (left, right) = lease.split(width / 2)?;
        for (mut child, x, count) in [(left, 0, width / 2), (right, width / 2, width - width / 2)] {
            for column in 0..count {
                child.put(column, color(x + column, row))?;
            }
        }
    }
    row_bytes.checked_mul(height).ok_or(Error::Bounds)
}
/// Validate every source before touching the destination. Each visible row copies
/// directly from its leaf; the number of ancestors does not add pixel passes.
pub fn compose(
    plan: &Plan,
    sources: &[Source<'_>; LEAVES],
    target: &mut [u8],
) -> Result<usize, Error> {
    let stride = plan.width.checked_mul(4).ok_or(Error::Bounds)?;
    if stride
        .checked_mul(plan.height)
        .is_none_or(|n| n > target.len())
    {
        return Err(Error::Bounds);
    }
    for (draw, source) in plan.draws.iter().zip(sources) {
        let Some(draw) = draw else {
            continue;
        };
        if draw.target.width == 0 || draw.target.height == 0 {
            continue;
        }
        let row = draw.source_width.checked_mul(4).ok_or(Error::Bounds)?;
        if source.stride < row
            || source
                .stride
                .checked_mul(draw.source_height)
                .is_none_or(|n| n > source.bytes.len())
        {
            return Err(Error::Bounds);
        }
        // Plans are public lab data: recheck bounds even if not made by resolve.
        if draw
            .source_x
            .checked_add(draw.target.width)
            .is_none_or(|n| n > draw.source_width)
            || draw
                .source_y
                .checked_add(draw.target.height)
                .is_none_or(|n| n > draw.source_height)
            || draw
                .target
                .x
                .checked_add(draw.target.width)
                .is_none_or(|n| n > plan.width)
            || draw
                .target
                .y
                .checked_add(draw.target.height)
                .is_none_or(|n| n > plan.height)
        {
            return Err(Error::Bounds);
        }
    }
    let mut copied = 0;
    for (draw, source) in plan.draws.iter().zip(sources) {
        let Some(draw) = draw else {
            continue;
        };
        if draw.target.width == 0 || draw.target.height == 0 {
            continue;
        }
        for row in 0..draw.target.height {
            let from = (draw.source_y + row) * source.stride + draw.source_x * 4;
            let to = (draw.target.y + row) * stride + draw.target.x * 4;
            let bytes = draw.target.width * 4;
            target[to..to + bytes].copy_from_slice(&source.bytes[from..from + bytes]);
            copied += bytes;
        }
    }
    Ok(copied)
}
pub fn verify(bytes: &[u8], width: usize, height: usize) -> bool {
    width != 0
        && height != 0
        && width
            .checked_mul(height)
            .and_then(|n| n.checked_mul(4))
            .is_some_and(|n| n == bytes.len())
        && bytes
            .chunks_exact(4)
            .enumerate()
            .all(|(i, p)| p == color(i % width, i / width).to_le_bytes())
}
