/*
 * Copyright (c) 2009, 2013, Oracle and/or its affiliates. All rights reserved.
 * Copyright 2009 Google Inc.  All Rights Reserved.
 * DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
 *
 * This code is free software; you can redistribute it and/or modify it
 * under the terms of the GNU General Public License version 2 only, as
 * published by the Free Software Foundation.  Oracle designates this
 * particular file as subject to the "Classpath" exception as provided
 * by Oracle in the LICENSE file that accompanied this code.
 *
 * This code is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
 * version 2 for more details (a copy is included in the LICENSE file that
 * accompanied this code).
 *
 * You should have received a copy of the GNU General Public License version
 * 2 along with this work; if not, write to the Free Software Foundation,
 * Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 * Please contact Oracle, 500 Oracle Parkway, Redwood Shores, CA 94065 USA
 * or visit www.oracle.com if you need additional information or have any
 * questions.
 */
// Rust adaptation of the local JDK TimSort for revision-239 comparator behavior.
//! Java-compatible stable object sorting, including comparator-contract failures.
//! Comparison order matters: game comparator chains need not define a total order.
use crate::{Error, Result};
use std::cmp::Ordering;
pub fn sort<T: Clone>(values: &mut [T], compare: impl Fn(&T, &T) -> Ordering) -> Result<()> {
    try_sort(values, |a, b| Ok(compare(a, b)))
}
pub fn try_sort<T: Clone>(
    values: &mut [T],
    compare: impl Fn(&T, &T) -> Result<Ordering>,
) -> Result<()> {
    let mut indices: Vec<usize> = (0..values.len()).collect();
    let status = sort_indices(&mut indices, &|a, b| compare(&values[a], &values[b]));
    let output: Vec<T> = indices.into_iter().map(|i| values[i].clone()).collect();
    values.clone_from_slice(&output);
    status
}
fn run(a: &mut [usize], lo: usize, c: &impl Fn(usize, usize) -> Result<Ordering>) -> Result<usize> {
    let mut end = lo + 1;
    if end == a.len() {
        return Ok(1);
    }
    let descending = c(a[end], a[lo])?.is_lt();
    end += 1;
    while end < a.len()
        && if descending {
            c(a[end], a[end - 1])?.is_lt()
        } else {
            !c(a[end], a[end - 1])?.is_lt()
        }
    {
        end += 1
    }
    if descending {
        a[lo..end].reverse()
    }
    Ok(end - lo)
}
fn binary(
    a: &mut [usize],
    lo: usize,
    hi: usize,
    start: usize,
    c: &impl Fn(usize, usize) -> Result<Ordering>,
) -> Result<()> {
    for i in start.max(lo + 1)..hi {
        let pivot = a[i];
        let (mut left, mut right) = (lo, i);
        while left < right {
            let mid = (left + right) / 2;
            if c(pivot, a[mid])?.is_lt() {
                right = mid
            } else {
                left = mid + 1
            }
        }
        a.copy_within(left..i, left + 1);
        a[left] = pivot;
    }
    Ok(())
}
fn gallop(
    key: usize,
    a: &[usize],
    base: usize,
    len: usize,
    hint: usize,
    right: bool,
    c: &impl Fn(usize, usize) -> Result<Ordering>,
) -> Result<usize> {
    let test = |value| -> Result<bool> {
        let order = c(key, value)?;
        Ok(if right { !order.is_lt() } else { order.is_gt() })
    };
    let (mut last, mut offset) = (0isize, 1isize);
    let h = hint as isize;
    if test(a[base + hint])? {
        let max = (len - hint) as isize;
        while offset < max && test(a[base + hint + offset as usize])? {
            last = offset;
            offset = 2 * offset + 1
        }
        offset = offset.min(max);
        last += h;
        offset += h;
    } else {
        let max = h + 1;
        while offset < max && !test(a[(base as isize + h - offset) as usize])? {
            last = offset;
            offset = 2 * offset + 1
        }
        offset = offset.min(max);
        let old = last;
        last = h - offset;
        offset = h - old;
    }
    last += 1;
    while last < offset {
        let mid = last + (offset - last) / 2;
        if test(a[base + mid as usize])? {
            last = mid + 1
        } else {
            offset = mid
        }
    }
    Ok(offset as usize)
}
fn sort_indices(a: &mut [usize], c: &impl Fn(usize, usize) -> Result<Ordering>) -> Result<()> {
    let n = a.len();
    if n < 2 {
        return Ok(());
    }
    if n < 32 {
        let initial = run(a, 0, c)?;
        binary(a, 0, n, initial, c)?;
        return Ok(());
    }
    let (mut min, mut remainder) = (n, 0);
    while min >= 32 {
        remainder |= min & 1;
        min >>= 1;
    }
    min += remainder;
    let mut stack: Vec<(usize, usize)> = Vec::new();
    let mut lo = 0;
    let mut gallop_min = 7;
    while lo < n {
        let mut size = run(a, lo, c)?;
        if size < min {
            let force = min.min(n - lo);
            binary(a, lo, lo + force, lo + size, c)?;
            size = force
        }
        stack.push((lo, size));
        lo += size;
        while stack.len() > 1 {
            let mut i = stack.len() - 2;
            if (i > 0 && stack[i - 1].1 <= stack[i].1 + stack[i + 1].1)
                || (i > 1 && stack[i - 2].1 <= stack[i].1 + stack[i - 1].1)
            {
                if stack[i - 1].1 < stack[i + 1].1 {
                    i -= 1
                }
            } else if stack[i].1 > stack[i + 1].1 {
                break;
            }
            merge(a, &mut stack, i, &mut gallop_min, c)?;
        }
    }
    while stack.len() > 1 {
        let mut i = stack.len() - 2;
        if i > 0 && stack[i - 1].1 < stack[i + 1].1 {
            i -= 1
        }
        merge(a, &mut stack, i, &mut gallop_min, c)?;
    }
    Ok(())
}
fn merge(
    a: &mut [usize],
    stack: &mut Vec<(usize, usize)>,
    i: usize,
    min: &mut i32,
    c: &impl Fn(usize, usize) -> Result<Ordering>,
) -> Result<()> {
    let (mut base1, mut len1) = stack[i];
    let (base2, mut len2) = stack[i + 1];
    stack[i].1 += len2;
    stack.remove(i + 1);
    let skip = gallop(a[base2], a, base1, len1, 0, true, c)?;
    base1 += skip;
    len1 -= skip;
    if len1 == 0 {
        return Ok(());
    }
    len2 = gallop(a[base1 + len1 - 1], a, base2, len2, len2 - 1, false, c)?;
    if len2 == 0 {
        return Ok(());
    }
    if len1 <= len2 {
        merge_low(a, base1, len1, base2, len2, min, c)
    } else {
        merge_high(a, base1, len1, base2, len2, min, c)
    }
}
fn merge_low(
    a: &mut [usize],
    base1: usize,
    mut n1: usize,
    base2: usize,
    mut n2: usize,
    min: &mut i32,
    c: &impl Fn(usize, usize) -> Result<Ordering>,
) -> Result<()> {
    let temp = a[base1..base1 + n1].to_vec();
    let (mut p1, mut p2, mut dst) = (0, base2, base1);
    a[dst] = a[p2];
    dst += 1;
    p2 += 1;
    n2 -= 1;
    if n2 == 0 {
        a[dst..dst + n1].copy_from_slice(&temp);
        return Ok(());
    }
    if n1 == 1 {
        a.copy_within(p2..p2 + n2, dst);
        a[dst + n2] = temp[p1];
        return Ok(());
    }
    let mut threshold = *min;
    'outer: loop {
        let (mut count1, mut count2) = (0, 0);
        loop {
            if c(a[p2], temp[p1])?.is_lt() {
                a[dst] = a[p2];
                dst += 1;
                p2 += 1;
                count2 += 1;
                count1 = 0;
                n2 -= 1;
                if n2 == 0 {
                    break 'outer;
                }
            } else {
                a[dst] = temp[p1];
                dst += 1;
                p1 += 1;
                count1 += 1;
                count2 = 0;
                n1 -= 1;
                if n1 == 1 {
                    break 'outer;
                }
            }
            if (count1 | count2) >= threshold {
                break;
            }
        }
        loop {
            let count1 = gallop(a[p2], &temp, p1, n1, 0, true, c)?;
            if count1 > 0 {
                a[dst..dst + count1].copy_from_slice(&temp[p1..p1 + count1]);
                dst += count1;
                p1 += count1;
                n1 -= count1;
                if n1 <= 1 {
                    break 'outer;
                }
            }
            a[dst] = a[p2];
            dst += 1;
            p2 += 1;
            n2 -= 1;
            if n2 == 0 {
                break 'outer;
            }
            let count2 = gallop(temp[p1], a, p2, n2, 0, false, c)?;
            if count2 > 0 {
                a.copy_within(p2..p2 + count2, dst);
                dst += count2;
                p2 += count2;
                n2 -= count2;
                if n2 == 0 {
                    break 'outer;
                }
            }
            a[dst] = temp[p1];
            dst += 1;
            p1 += 1;
            n1 -= 1;
            if n1 == 1 {
                break 'outer;
            }
            threshold -= 1;
            if count1 < 7 && count2 < 7 {
                break;
            }
        }
        threshold = threshold.max(0) + 2;
    }
    *min = threshold.max(1);
    if n1 == 1 {
        a.copy_within(p2..p2 + n2, dst);
        a[dst + n2] = temp[p1];
    } else if n1 == 0 {
        return Err(Error::Invalid("Java comparator contract"));
    } else {
        a[dst..dst + n1].copy_from_slice(&temp[p1..p1 + n1]);
    }
    Ok(())
}
fn merge_high(
    a: &mut [usize],
    base1: usize,
    mut n1: usize,
    base2: usize,
    mut n2: usize,
    min: &mut i32,
    c: &impl Fn(usize, usize) -> Result<Ordering>,
) -> Result<()> {
    let temp = a[base2..base2 + n2].to_vec();
    let (mut p1, mut p2, mut dst) = (
        (base1 + n1 - 1) as isize,
        (n2 - 1) as isize,
        (base2 + n2 - 1) as isize,
    );
    a[dst as usize] = a[p1 as usize];
    dst -= 1;
    p1 -= 1;
    n1 -= 1;
    if n1 == 0 {
        let start = dst as usize + 1 - n2;
        a[start..start + n2].copy_from_slice(&temp);
        return Ok(());
    }
    if n2 == 1 {
        dst -= n1 as isize;
        p1 -= n1 as isize;
        a.copy_within(
            (p1 + 1) as usize..(p1 + 1) as usize + n1,
            (dst + 1) as usize,
        );
        a[dst as usize] = temp[p2 as usize];
        return Ok(());
    }
    let mut threshold = *min;
    'outer: loop {
        let (mut count1, mut count2) = (0, 0);
        loop {
            if c(temp[p2 as usize], a[p1 as usize])?.is_lt() {
                a[dst as usize] = a[p1 as usize];
                dst -= 1;
                p1 -= 1;
                count1 += 1;
                count2 = 0;
                n1 -= 1;
                if n1 == 0 {
                    break 'outer;
                }
            } else {
                a[dst as usize] = temp[p2 as usize];
                dst -= 1;
                p2 -= 1;
                count2 += 1;
                count1 = 0;
                n2 -= 1;
                if n2 == 1 {
                    break 'outer;
                }
            }
            if (count1 | count2) >= threshold {
                break;
            }
        }
        loop {
            let count1 = n1 - gallop(temp[p2 as usize], a, base1, n1, n1 - 1, true, c)?;
            if count1 > 0 {
                dst -= count1 as isize;
                p1 -= count1 as isize;
                n1 -= count1;
                a.copy_within(
                    (p1 + 1) as usize..(p1 + 1) as usize + count1,
                    (dst + 1) as usize,
                );
                if n1 == 0 {
                    break 'outer;
                }
            }
            a[dst as usize] = temp[p2 as usize];
            dst -= 1;
            p2 -= 1;
            n2 -= 1;
            if n2 == 1 {
                break 'outer;
            }
            let count2 = n2 - gallop(a[p1 as usize], &temp, 0, n2, n2 - 1, false, c)?;
            if count2 > 0 {
                dst -= count2 as isize;
                p2 -= count2 as isize;
                n2 -= count2;
                a[(dst + 1) as usize..(dst + 1) as usize + count2]
                    .copy_from_slice(&temp[(p2 + 1) as usize..(p2 + 1) as usize + count2]);
                if n2 <= 1 {
                    break 'outer;
                }
            }
            a[dst as usize] = a[p1 as usize];
            dst -= 1;
            p1 -= 1;
            n1 -= 1;
            if n1 == 0 {
                break 'outer;
            }
            threshold -= 1;
            if count1 < 7 && count2 < 7 {
                break;
            }
        }
        threshold = threshold.max(0) + 2;
    }
    *min = threshold.max(1);
    if n2 == 1 {
        dst -= n1 as isize;
        p1 -= n1 as isize;
        a.copy_within(
            (p1 + 1) as usize..(p1 + 1) as usize + n1,
            (dst + 1) as usize,
        );
        a[dst as usize] = temp[p2 as usize];
    } else if n2 == 0 {
        return Err(Error::Invalid("Java comparator contract"));
    } else {
        let start = (dst + 1) as usize - n2;
        a[start..start + n2].copy_from_slice(&temp[..n2]);
    }
    Ok(())
}
