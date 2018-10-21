use std;
//
// Sweep-and-update Euclidean distance transform of an
// image. Positive pixels are treated as object pixels,
// zero or negative pixels are treated as background.
// An attempt is made to treat antialiased edges correctly.
// The input image must have pixels in the range [0,1],
// and the antialiased image should be a box-filter
// sampling of the ideal, crisp edge.
// If the antialias region is more than 1 pixel wide,
// the result from this transform will be inaccurate.
//

//
// Compute the local gradient at edge pixels using convolution filters.
// The gradient is computed only at edge pixels. At other places in the
// image, it is never used, and it's mostly zero anyway.
//
fn computegradient(img: &[f64], w: usize, h: usize, gx: &mut [f64], gy: &mut [f64]) {
    let mut glength;
    const SQRT2: f64 = 1.4142136;
    //for(i = 1; i < h-1; i++) // Avoid edges where the kernels would spill over
    for i in 1..(h - 1) {
        //for(j = 1; j < w-1; j++)
        for j in 1..(w - 1) {
            let k = i * w + j;
            if (img[k] > 0.0) && (img[k] < 1.0) {
                // Compute gradient for edge pixels only
                gx[k] = -img[k - w - 1] - SQRT2 * img[k - 1] - img[k + w - 1]
                    + img[k - w + 1]
                    + SQRT2 * img[k + 1]
                    + img[k + w + 1];
                gy[k] = -img[k - w - 1] - SQRT2 * img[k - w] - img[k - w + 1]
                    + img[k + w - 1]
                    + SQRT2 * img[k + w]
                    + img[k + w + 1];
                glength = gx[k] * gx[k] + gy[k] * gy[k];
                if glength > 0.0 {
                    // Avoid division by zero
                    glength = glength.sqrt();
                    gx[k] = gx[k] / glength;
                    gy[k] = gy[k] / glength;
                }
            }
        }
    }
    // TODO: Compute reasonable values for gx, gy also around the image edges.
    // (These are zero now, which reduces the accuracy for a 1-pixel wide region
    // around the image edge.) 2x2 kernels would be suitable for this.
}

//
// A somewhat tricky function to approximate the distance to an edge in a
// certain pixel, with consideration to either the local gradient (gx,gy)
// or the direction to the pixel (dx,dy) and the pixel greyscale value a.
// The latter alternative, using (dx,dy), is the metric used by edtaa2().
// Using a local estimate of the edge gradient (gx,gy) yields much better
// accuracy at and near edges, and reduces the error even at distant pixels
// provided that the gradient direction is accurately estimated.
//

fn edgedf(mut gx: f64, mut gy: f64, a: f64) -> f64 {
    let (df, glength, temp, a1);

    if (gx == 0.0) || (gy == 0.0) {
        // Either A) gu or gv are zero, or B) both
        df = 0.5 - a; // Linear approximation is A) correct or B) a fair guess
    } else {
        glength = (gx * gx + gy * gy).sqrt();
        if glength > 0.0 {
            gx = gx / glength;
            gy = gy / glength;
        }
        /* Everything is symmetric wrt sign and transposition,
         * so move to first octant (gx>=0, gy>=0, gx>=gy) to
         * avoid handling all possible edge directions. */
        gx = gx.abs();
        gy = gy.abs();
        if gx < gy {
            temp = gx;
            gx = gy;
            gy = temp;
        }
        a1 = 0.5 * gy / gx;
        if a < a1 {
            // 0 <= a < a1
            df = 0.5 * (gx + gy) - (2.0 * gx * gy * a).sqrt();
        } else if a < (1.0 - a1) {
            // a1 <= a <= 1-a1
            df = (0.5 - a) * gx;
        } else {
            // 1-a1 < a <= 1
            df = -0.5 * (gx + gy) + (2.0 * gx * gy * (1.0 - a)).sqrt();
        }
    }
    df
}

fn distaa3(
    img: &[f64],
    gximg: &[f64],
    gyimg: &[f64],
    w: i32,
    c: i32,
    xc: i32,
    yc: i32,
    xi: i32,
    yi: i32,
) -> f64 {
    let (di, df, dx, dy, gx, gy, mut a);
    let closest;

    closest = (c - xc - yc * w) as usize; // Index to the edge pixel pointed to from c
    a = img[closest]; // Grayscale value at the edge pixel
    gx = gximg[closest]; // X gradient component at the edge pixel
    gy = gyimg[closest]; // Y gradient component at the edge pixel

    if a > 1.0 {
        a = 1.0;
    }
    if a < 0.0 {
        a = 0.0; // Clip grayscale values outside the range [0,1]
    }
    if a == 0.0 {
        return 1000000.0; // Not an object pixel, return "very far" ("don't know yet")
    }

    dx = xi as f64;
    dy = yi as f64;
    di = (dx * dx + dy * dy).sqrt(); // Length of integer vector, like a traditional EDT
    if di == 0.0 {
        // Use local gradient only at edges
        // Estimate based on local gradient only
        df = edgedf(gx, gy, a);
    } else {
        // Estimate gradient based on direction to edge (accurate for large di)
        df = edgedf(dx, dy, a);
    }
    return di + df; // Same metric as edtaa2, except at edges (where di=0)
}

//
// Shorthand macro: add ubiquitous parameters dist, gx, gy, img and w and call distaa3()
// #define DISTAA(c,xc,yc,xi,yi) (distaa3(img, gx, gy, w, c, xc, yc, xi, yi))
//

fn edtaa3(
    img: &[f64],
    gx: &[f64],
    gy: &[f64],
    w: isize,
    h: isize,
    distx: &mut [i16],
    disty: &mut [i16],
    dist: &mut [f64],
) {
    let (mut i, mut c);
    let (offset_u, offset_ur, offset_r, offset_rd, offset_d, offset_dl, offset_l, offset_lu);
    let (mut olddist, mut newdist);
    let (mut cdistx, mut cdisty, mut newdistx, mut newdisty);
    let mut changed;
    let epsilon: f64 = 1e-3;

    /* Initialize index offsets for the current image width */
    offset_u = -w;
    offset_ur = -w + 1;
    offset_r = 1 as isize;
    offset_rd = w + 1;
    offset_d = w;
    offset_dl = w - 1;
    offset_l = -1 as isize;
    offset_lu = -w - 1;

    /* Initialize the distance images */
    //for(i=0; i<w*h; i++)
    for i in 0..((w * h) as usize) {
        distx[i] = 0; // At first, all pixels point to
        disty[i] = 0; // themselves as the closest known.
        if img[i] <= 0.0 {
            dist[i] = 1000000.0; // Big value, means "not set yet"
        } else if img[i] < 1.0 {
            dist[i] = edgedf(gx[i], gy[i], img[i]); // Gradient-assisted estimate
        } else {
            dist[i] = 0.0; // Inside the object
        }
    }

    /* Perform the transformation */
    loop {
        changed = 0;

        /* Scan rows, except first row */
        //for(y=1; y<h; y++)
        for y in 1..h {
            /* move index to leftmost pixel of current row */
            i = (y * w) as usize;

            /* scan right, propagate distances from above & left */

            /* Leftmost pixel is special, has no left neighbors */
            olddist = dist[i];
            if olddist > 0.0
            // If non-zero distance or not set yet
            {
                c = (i as isize + offset_u) as usize; // Index of candidate for testing
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx;
                newdisty = cdisty + 1;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    olddist = newdist;
                    changed = 1;
                }

                c = (i as isize + offset_ur) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx - 1;
                newdisty = cdisty + 1;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    changed = 1;
                }
            }
            i += 1;

            /* Middle pixels have all neighbors */
            //for(x=1; x<w-1; x++, i++)
            for _x in 1..(w - 1) {
                olddist = dist[i];
                if olddist <= 0.0 {
                    i += 1; // Extra
                    continue; // No need to update further
                }

                c = (i as isize + offset_l) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx + 1;
                newdisty = cdisty;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    olddist = newdist;
                    changed = 1;
                }

                c = (i as isize + offset_lu) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx + 1;
                newdisty = cdisty + 1;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    olddist = newdist;
                    changed = 1;
                }

                c = (i as isize + offset_u) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx;
                newdisty = cdisty + 1;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    olddist = newdist;
                    changed = 1;
                }

                c = (i as isize + offset_ur) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx - 1;
                newdisty = cdisty + 1;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    changed = 1;
                }
                i += 1; // Extra
            }

            /* Rightmost pixel of row is special, has no right neighbors */
            olddist = dist[i];
            if olddist > 0.0
            // If not already zero distance
            {
                c = (i as isize + offset_l) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx + 1;
                newdisty = cdisty;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    olddist = newdist;
                    changed = 1;
                }

                c = (i as isize + offset_lu) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx + 1;
                newdisty = cdisty + 1;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    olddist = newdist;
                    changed = 1;
                }

                c = (i as isize + offset_u) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx;
                newdisty = cdisty + 1;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    changed = 1;
                }
            }

            /* Move index to second rightmost pixel of current row. */
            /* Rightmost pixel is skipped, it has no right neighbor. */
            i = (y * w + w - 2) as usize;

            /* scan left, propagate distance from right */
            //for(x=w-2; x>=0; x--, i--)
            for _x in (0..=(w - 2)).rev() {
                olddist = dist[i];
                if olddist <= 0.0 {
                    i -= 1; // Extra
                    continue; // Already zero distance
                }

                c = (i as isize + offset_r) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx - 1;
                newdisty = cdisty;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    changed = 1;
                }
                i -= 1; // Extra
            }
        }

        /* Scan rows in reverse order, except last row */
        //for(y=h-2; y>=0; y--)
        for y in (0..=(h - 2)).rev() {
            /* move index to rightmost pixel of current row */
            i = (y * w + w - 1) as usize;

            /* Scan left, propagate distances from below & right */

            /* Rightmost pixel is special, has no right neighbors */
            olddist = dist[i];
            if olddist > 0.0
            // If not already zero distance
            {
                c = (i as isize + offset_d) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx;
                newdisty = cdisty - 1;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    olddist = newdist;
                    changed = 1;
                }

                c = (i as isize + offset_dl) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx + 1;
                newdisty = cdisty - 1;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    changed = 1;
                }
            }
            i -= 1;

            /* Middle pixels have all neighbors */
            //for(x=w-2; x>0; x--, i--)
            for _x in 1..=(w - 2) {
                olddist = dist[i];
                if olddist <= 0.0 {
                    i -= 1; // Extra
                    continue; // Already zero distance
                }

                c = (i as isize + offset_r) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx - 1;
                newdisty = cdisty;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    olddist = newdist;
                    changed = 1;
                }

                c = (i as isize + offset_rd) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx - 1;
                newdisty = cdisty - 1;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    olddist = newdist;
                    changed = 1;
                }

                c = (i as isize + offset_d) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx;
                newdisty = cdisty - 1;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    olddist = newdist;
                    changed = 1;
                }

                c = (i as isize + offset_dl) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx + 1;
                newdisty = cdisty - 1;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    changed = 1;
                }
                i -= 1; // Extra
            }
            /* Leftmost pixel is special, has no left neighbors */
            olddist = dist[i];
            if olddist > 0.0
            // If not already zero distance
            {
                c = (i as isize + offset_r) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx - 1;
                newdisty = cdisty;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    olddist = newdist;
                    changed = 1;
                }

                c = (i as isize + offset_rd) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx - 1;
                newdisty = cdisty - 1;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    olddist = newdist;
                    changed = 1;
                }

                c = (i as isize + offset_d) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx;
                newdisty = cdisty - 1;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    changed = 1;
                }
            }

            /* Move index to second leftmost pixel of current row. */
            /* Leftmost pixel is skipped, it has no left neighbor. */
            i = (y * w + 1) as usize;
            //for(x=1; x<w; x++, i++)
            for _x in 1..w {
                /* scan right, propagate distance from left */
                olddist = dist[i];
                if olddist <= 0.0 {
                    i += 1; // Extra
                    continue; // Already zero distance
                }

                c = (i as isize + offset_l) as usize;
                cdistx = distx[c];
                cdisty = disty[c];
                newdistx = cdistx + 1;
                newdisty = cdisty;
                newdist = distaa3(
                    &img,
                    &gx,
                    &gy,
                    w as i32,
                    c as i32,
                    cdistx.into(),
                    cdisty.into(),
                    newdistx.into(),
                    newdisty.into(),
                );
                if newdist < olddist - epsilon {
                    distx[i] = newdistx;
                    disty[i] = newdisty;
                    dist[i] = newdist;
                    changed = 1;
                }
                i += 1; // Extra
            }
        }

        // Sweep until no more updates are made
        if changed == 0 {
            break;
        }
    }
    /* The transformation is completed. */
}

/* Create a distance map from the given grayscale image.
 * Returns a newly allocated distance field. This image must
 * be freed after usage. */
pub fn make_distance_mapd(data: &mut [f64], width: usize, height: usize) {
    let mut xdist = vec![0i16; width * height];
    let mut ydist = vec![0i16; width * height];
    let mut gx = vec![0.0; width * height];
    let mut gy = vec![0.0; width * height];
    let mut outside = vec![0.0; width * height];
    let mut inside = vec![0.0; width * height];
    let mut vmin = std::f64::MAX;

    /* Compute outside = edtaa3(bitmap); % Transform background (0's) */
    computegradient(data, width, height, &mut gx, &mut gy);
    edtaa3(
        data,
        &mut gx,
        &mut gy,
        width as isize,
        height as isize,
        &mut xdist,
        &mut ydist,
        &mut outside,
    );
    for i in 0..(width * height) {
        if outside[i] < 0.0 {
            outside[i] = 0.0;
        }
    }

    /* Compute inside = edtaa3(1-bitmap); % Transform foreground (1's) */
    gx.clear();
    gx.resize(width * height, 0.0);
    gy.clear();
    gy.resize(width * height, 0.0);
    gx = vec![0.0; width * height];
    gy = vec![0.0; width * height];
    for i in 0..(width * height) {
        data[i] = 1.0 - data[i];
    }
    computegradient(data, width, height, &mut gx, &mut gy);
    edtaa3(
        data,
        &mut gx,
        &mut gy,
        width as isize,
        height as isize,
        &mut xdist,
        &mut ydist,
        &mut inside,
    );
    for i in 0..(width * height) {
        if inside[i] < 0.0 {
            inside[i] = 0.0;
        }
    }

    /* distmap = outside - inside; % Bipolar distance field */
    for i in 0..(width * height) {
        outside[i] -= inside[i];
        if outside[i] < vmin {
            vmin = outside[i];
        }
    }

    vmin = vmin.abs();

    for i in 0..(width * height) {
        let v = outside[i];
        if v < -vmin {
            outside[i] = -vmin;
        } else if v > vmin {
            outside[i] = vmin;
        }
        data[i] = (outside[i] + vmin) / (2.0 * vmin);
    }
}

#[allow(dead_code)]
pub fn make_distance_mapb(img: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut data = vec![0.0; width * height];
    let mut out = vec![0u8; width * height];

    /* Find minimimum and maximum values */
    let mut img_min = std::f64::MAX;
    let mut img_max = std::f64::MIN;

    for i in 0..(width * height) {
        let v = img[i] as f64;
        data[i] = v;
        if v > img_max {
            img_max = v;
        }
        if v < img_min {
            img_min = v;
        }
    }

    /* Map values from 0 - 255 to 0.0 - 1.0 */
    for i in 0..(width * height) {
        data[i] = (img[i] as f64 - img_min) / img_max;
    }

    make_distance_mapd(&mut data, width, height);

    /* Map values from 0.0 - 1.0 to 0 - 255 */
    for i in 0..(width * height) {
        out[i] = ((255.0 * (1.0 - data[i])) as u64) as u8;
    }

    out
}
