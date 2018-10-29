use super::sdf;
use super::shader::*;
use gl;
use gl::types::*;
use rusttype::gpu_cache::Cache;
use rusttype::{point, Font, PositionedGlyph, Rect, Scale};
use std;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::BufRead;
use std::mem::size_of;

const FONT_LOAD_SIZE: f32 = 48.0;

const VERTEX_SHADER: &str = "\
#version 300 es
in vec2 vpos;
in vec2 vtco;

out vec2 tco;
uniform mat4 mvp;

void main()
{
    tco = vtco;
    gl_Position = mvp * vec4(vpos, 0.0, 1.0);
}
";

const FRAGMENT_SHADER: &str = "\
#version 300 es

#ifdef GL_OES_standard_derivatives
#extension GL_OES_standard_derivatives : enable
const bool HAS_DERIVATIVES = true;
#else
const bool HAS_DERIVATIVES = false;
#endif

#ifdef GL_ES
precision mediump float;
#endif

out vec4 fcolor;
in vec2 tco;

uniform vec4 col;
uniform float scl;
uniform sampler2D tex;
uniform bool ssp;
uniform bool dfd;

const float SQRT2_2 = 0.70710678118654757;

float contour(float d, float w)
{
    return smoothstep(0.5 - w, 0.5 + w, d);
}

void main()
{
    vec2 uv = tco;
    float dist = texture(tex, uv).a;

    // Keep outlines a constant width irrespective of scaling
    float fw = 0.0;
    if (dfd && HAS_DERIVATIVES) {
        // GLSL's fwidth = abs(dFdx(dist)) + abs(dFdy(dist))
        fw = fwidth(dist);
        // Stefan Gustavson's fwidth
        //fw = SQRT2_2 * length(vec2(dFdx(dist), dFdy(dist)));
    } else {
        fw = (1.0 / scl) * SQRT2_2 / gl_FragCoord.w;
    }
    float alpha = contour(dist, fw);

    if (ssp) {
        // Supersample
        float dscale = 0.354; // half of 1/sqrt2
        vec2 duv = dscale * (dFdx(uv) + dFdy(uv));
        vec4 box = vec4(uv - duv, uv + duv);
        float asum = contour(texture(tex, box.xy).a, fw)
                   + contour(texture(tex, box.zw).a, fw)
                   + contour(texture(tex, box.xw).a, fw)
                   + contour(texture(tex, box.zy).a, fw);
        // Weighted average, with 4 extra points having 0.5 weight each,
        // so 1 + 0.5 * 4 = 3 is the divisor
        alpha = (alpha + 0.5 * asum) / 3.0;
    }

    fcolor = col * vec4(vec3(1.0), alpha);
}
";

struct Vertex([f32; 2], [f32; 2]);

pub struct TextRenderer {
    font_id_gen: usize,
    font_map: HashMap<String, (usize, Font<'static>)>,
    cache: RefCell<Cache<'static>>,
    cache_img_id: GLuint,
    shader: Shader,
    draw_vbo: GLuint,
    draw_ebo: GLuint,
}

#[allow(dead_code)]
pub enum HAlignment {
    Left,
    Center,
    Right,
}

#[allow(dead_code)]
pub enum VAlignment {
    Top,
    Center,
    Bottom,
}

pub struct Text<'a> {
    contents: &'a str,
    font: &'a str,
    transform: &'a [[f32; 4]; 4],
    color: [f32; 4],
    halign: HAlignment,
    valign: VAlignment,
    use_vmetrics: bool,
    dfd_antialiasing: bool,
    super_sample: bool,
}

#[allow(dead_code)]
impl <'a> Text<'a> {
    pub fn new(contents: &'a str, font: &'a str, transform: &'a [[f32; 4]; 4]) -> Self {
        Text {
            contents,
            font,
            transform,
            color: [1.0; 4],
            halign: HAlignment::Center,
            valign: VAlignment::Center,
            use_vmetrics: false,
            dfd_antialiasing: false,
            super_sample: true,
        }
    }

    pub fn with_color(mut self, color: &[f32; 4]) -> Self {
        self.color = *color;
        self
    }

    pub fn with_halignment(mut self, halign: HAlignment) -> Self {
        self.halign = halign;
        self
    }

    pub fn with_valignment(mut self, valign: VAlignment) -> Self {
        self.valign = valign;
        self
    }

    pub fn with_use_vmetrics(mut self, use_vmetrics: bool) -> Self {
        self.use_vmetrics = use_vmetrics;
        self
    }

    pub fn with_super_sample(mut self, super_sample: bool) -> Self {
        self.super_sample = super_sample;
        self
    }

    pub fn with_dfd_antialiasing(mut self, dfd_antialiasing: bool) -> Self {
        self.dfd_antialiasing = dfd_antialiasing;
        self
    }

    pub fn draw(&self, rndr: &TextRenderer) {
        rndr.draw(self)
    }
}

impl TextRenderer {
    pub fn new() -> Self {
        // Make gpu cache
        let (cache_width, cache_height) = (512, 512);
        let cache = Cache::builder()
            .dimensions(cache_width, cache_height)
            .build();

        // Make font atlas texture (GPU)
        let mut id: GLuint = 0;
        let null_data = vec![0u8; (cache_width * cache_height) as usize];
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::ALPHA as _,
                cache_width as _,
                cache_height as _,
                0,
                gl::ALPHA,
                gl::UNSIGNED_BYTE,
                null_data.as_ptr() as _,
            );
        }

        // Compile shader
        let shdr = Shader::new(
            VERTEX_SHADER,
            None,
            FRAGMENT_SHADER,
            Some(&["vpos", "vnrm", "vuv0"]),
        );

        // Make draw buffers
        let mut vbo: GLuint = 0;
        let mut ebo: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
        }

        TextRenderer {
            font_id_gen: 0,
            font_map: HashMap::new(),
            cache: RefCell::new(cache),
            cache_img_id: id,
            shader: shdr,
            draw_vbo: vbo,
            draw_ebo: ebo,
        }
    }

    pub fn add_font<B: BufRead>(&mut self, name: &str, reader: &mut B) {
        // Load font from data
        let mut font_data = Vec::new();
        reader.read_to_end(&mut font_data).unwrap();
        let font = Font::from_bytes(font_data).unwrap();
        // Add to map
        self.font_map
            .insert(name.to_string(), (self.font_id_gen, font));
        self.font_id_gen += 1;
    }

    pub fn draw(
        &self,
        t: &Text
    ) {
        // Find font
        let (font_id, font) = match self.font_map.get(t.font) {
            Some(a) => a,
            None => return,
        };

        // Get gluphs
        let (glyphs, num_lines) =
            self.layout_paragraph(font, Scale::uniform(FONT_LOAD_SIZE), 2000, t.contents);

        // Queue some positioned glyphs needed for the next frame
        for glyph in &glyphs {
            self.cache.borrow_mut().queue_glyph(*font_id, glyph.clone());
        }

        // Cache all queued glyphs somewhere in the cache texture.
        // If new glyph data has been drawn the closure is called to upload
        // the pixel data to GPU memory.
        self.cache
            .borrow_mut()
            .cache_queued(|region, data| {
                // Pad data
                let (rw, rh) = (region.width() as usize, region.height() as usize);
                let pad = 0; // TODO: make padding 1
                let (nw, nh) = (rw + pad, rh + pad);
                let mut padded_data = vec![0u8; nw * nh];
                for i in 0..(nh - pad) {
                    let src = &data[(i * rw)..((i + 1) * rw)];
                    let dst = &mut padded_data[(i * nw)..((i + 1) * nw - pad)];
                    dst.copy_from_slice(src);
                }
                // Make Signed Distance Field
                let dist_map = sdf::make_distance_mapb(&mut padded_data, nw, nh);
                // Update GPU texture
                unsafe {
                    // Update part of gpu texture with new glyph alpha values
                    gl::BindTexture(gl::TEXTURE_2D, self.cache_img_id);
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                    gl::TexSubImage2D(
                        gl::TEXTURE_2D,
                        0,
                        region.min.x as _,
                        region.min.y as _,
                        nw as _,
                        nh as _,
                        gl::ALPHA,
                        gl::UNSIGNED_BYTE,
                        dist_map.as_ptr() as _,
                    );
                    gl::BindTexture(gl::TEXTURE_2D, 0);
                }
            }).unwrap();

        // Build vertex and indice data
        let (mut vertices, indices) = self.build_vertex_and_indice_data(&glyphs, *font_id);

        // Get viewport size
        let vp: [GLint; 4] = [0; 4];
        unsafe {
            gl::GetIntegerv(gl::VIEWPORT, vp.as_ptr() as *mut GLint);
        }
        let (scr_w, scr_h) = ((vp[2] - vp[0]) as f32, (vp[3] - vp[1]) as f32);

        // Get phrase bounding box
        let bbox = vertices.iter().fold(
            Rect {
                min: point(std::f32::MAX, std::f32::MAX),
                max: point(-std::f32::MAX, -std::f32::MAX),
            },
            |mut acc, x| {
                let vpos = x.0;
                acc.min.x = acc.min.x.min(vpos[0]);
                acc.max.x = acc.max.x.max(vpos[0]);
                acc.min.y = acc.min.y.min(vpos[1]);
                acc.max.y = acc.max.y.max(vpos[1]);
                acc
            },
        );

        // Alignment
        let v_metrics = font.v_metrics(Scale::uniform(FONT_LOAD_SIZE));
        for v in vertices.iter_mut() {
            // Center in bbox horizontally
            v.0[0] -= bbox.min.x + bbox.width() / 2.0;
            // Flip y
            v.0[1] = -v.0[1];
            // Horizontal alignment
            match t.halign {
                HAlignment::Left => v.0[0] -= bbox.width() / 2.0,
                HAlignment::Center => (),
                HAlignment::Right => v.0[0] += bbox.width() / 2.0,
            }
            // Vertical alignment
            if !t.use_vmetrics {
                // Center in bbox vertically
                v.0[1] += bbox.min.y + bbox.height() / 2.0;
                match t.valign {
                    VAlignment::Top => v.0[1] += bbox.height() / 2.0,
                    VAlignment::Center => (),
                    VAlignment::Bottom => v.0[1] -= bbox.height() / 2.0,
                }
            } else {
                let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
                match t.valign {
                    VAlignment::Top => {
                        v.0[1] += num_lines as f32 * advance_height;
                    }
                    VAlignment::Center => {
                        v.0[1] += bbox.min.y + bbox.height() / 2.0;
                    }
                    VAlignment::Bottom => {
                        v.0[1] -= v_metrics.descent;
                    }
                }
            }
            // Normalize
            v.0[0] = (v.0[0] / scr_w) * 2.0;
            v.0[1] = (v.0[1] / scr_h) * 2.0;
            // Scale (convert to em)
            let fscale = 16.0 / FONT_LOAD_SIZE;
            v.0[0] *= fscale;
            v.0[1] *= fscale;
        }

        unsafe {
            // Upload data
            gl::BindBuffer(gl::ARRAY_BUFFER, self.draw_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * size_of::<Vertex>()) as GLsizeiptr,
                vertices.as_ptr() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.draw_ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * size_of::<u32>()) as GLsizeiptr,
                indices.as_ptr() as *const GLvoid,
                gl::DYNAMIC_DRAW,
            );

            // Setup attribute bindings
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as GLint,
                0 as *const GLvoid,
            );
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Vertex>() as GLint,
                (2 * size_of::<f32>()) as *const GLvoid,
            );

            // Compute scale factor
            let m = &t.transform;
            let scl = (m[1][1] * m[1][1] + m[1][2] * m[1][2] + m[1][3] * m[1][3]).sqrt();

            // Draw
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.cache_img_id);
            self.shader.activate();
            self.shader.set_uniform("col", &t.color);
            self.shader.set_uniform("mvp", t.transform);
            self.shader.set_uniform("ssp", t.super_sample);
            self.shader.set_uniform("dfd", t.dfd_antialiasing);
            self.shader.set_uniform("scl", scl);
            self.shader.set_uniform("tex", 0);
            gl::DrawElements(
                gl::TRIANGLES,
                indices.len() as GLint,
                gl::UNSIGNED_INT,
                0 as *const GLvoid,
            );
            gl::Disable(gl::BLEND);
        }
    }

    fn build_vertex_and_indice_data(
        &self,
        glyphs: &[PositionedGlyph],
        font_id: usize,
    ) -> (Vec<Vertex>, Vec<u32>) {
        let mut nglyphs = 0;
        let vertices: Vec<_> = glyphs
            .iter()
            .flat_map(|g| {
                // Lookup a positioned glyph's texture location
                if let Ok(Some((uv_rect, scr_rect))) = self.cache.borrow().rect_for(font_id, g) {
                    nglyphs += 1;
                    let sc_rect = Rect {
                        min: point(scr_rect.min.x as f32, scr_rect.min.y as f32),
                        max: point(scr_rect.max.x as f32, scr_rect.max.y as f32),
                    };
                    let verts = vec![
                        Vertex(
                            [sc_rect.min.x, sc_rect.min.y],
                            [uv_rect.min.x, uv_rect.min.y],
                        ),
                        Vertex(
                            [sc_rect.min.x, sc_rect.max.y],
                            [uv_rect.min.x, uv_rect.max.y],
                        ),
                        Vertex(
                            [sc_rect.max.x, sc_rect.max.y],
                            [uv_rect.max.x, uv_rect.max.y],
                        ),
                        Vertex(
                            [sc_rect.max.x, sc_rect.min.y],
                            [uv_rect.max.x, uv_rect.min.y],
                        ),
                    ];
                    verts
                } else {
                    vec![]
                }
            }).collect();
        let indices: Vec<u32> = (0..nglyphs)
            .flat_map(|i| [0, 1, 2, 0, 2, 3].iter().map(move |x| x + (i as u32) * 4))
            .collect();
        (vertices, indices)
    }

    fn layout_paragraph(
        &self,
        font: &Font<'static>,
        scale: Scale,
        width: u32,
        text: &str,
    ) -> (Vec<PositionedGlyph<'static>>, u32) {
        let mut result = Vec::new();
        let v_metrics = font.v_metrics(scale);
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        let mut caret = point(0.0, v_metrics.ascent);
        let mut num_lines = 1;
        let mut last_glyph_id = None;
        for c in text.chars() {
            if c.is_control() {
                match c {
                    '\r' => {
                        caret = point(0.0, caret.y + advance_height);
                        num_lines += 1;
                    }
                    '\n' => {}
                    _ => {}
                }
                continue;
            }
            let base_glyph = font.glyph(c);
            if let Some(id) = last_glyph_id.take() {
                caret.x += font.pair_kerning(scale, id, base_glyph.id());
            }
            last_glyph_id = Some(base_glyph.id());
            let mut glyph = base_glyph.scaled(scale).positioned(caret);
            if let Some(bb) = glyph.pixel_bounding_box() {
                if bb.max.x > width as i32 {
                    caret = point(0.0, caret.y + advance_height);
                    glyph = glyph.into_unpositioned().positioned(caret);
                    last_glyph_id = None;
                }
            }
            caret.x += glyph.unpositioned().h_metrics().advance_width;
            result.push(glyph);
        }
        (result, num_lines)
    }
}

impl Drop for TextRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.cache_img_id);
            gl::DeleteBuffers(1, &self.draw_ebo);
            gl::DeleteBuffers(1, &self.draw_vbo);
        }
    }
}
