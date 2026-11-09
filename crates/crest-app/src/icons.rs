use cef::{AlphaType, ColorType, Image, ImplImage, image_create};
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Rect, Stroke, Transform};

#[derive(Clone, Copy)]
pub(crate) enum Icon {
    Crest,
    Back,
    Forward,
    Reload,
    Home,
    Search,
    Star,
    Starred,
    Library,
    Download,
    Layout,
    Menu,
    Plus,
    Close,
    Tab,
}

pub(crate) fn render(icon: Icon, color: u32) -> Option<Image> {
    let image = image_create()?;
    let mut path = PathBuilder::new();
    let filled = match icon {
        Icon::Crest => {
            line(
                &mut path,
                &[
                    (3.0, 17.5),
                    (10.0, 6.5),
                    (14.0, 13.0),
                    (17.0, 9.0),
                    (21.0, 17.5),
                    (3.0, 17.5),
                ],
            );
            path.close();
            false
        }
        Icon::Back => {
            line(&mut path, &[(19.0, 12.0), (5.0, 12.0)]);
            line(&mut path, &[(11.0, 6.0), (5.0, 12.0), (11.0, 18.0)]);
            false
        }
        Icon::Forward => {
            line(&mut path, &[(5.0, 12.0), (19.0, 12.0)]);
            line(&mut path, &[(13.0, 6.0), (19.0, 12.0), (13.0, 18.0)]);
            false
        }
        Icon::Reload => {
            path.move_to(19.4, 9.0);
            path.cubic_to(17.8, 4.9, 12.8, 3.5, 9.1, 5.0);
            path.cubic_to(5.3, 6.6, 3.4, 11.1, 5.0, 14.8);
            path.cubic_to(6.6, 18.8, 11.2, 20.6, 15.1, 19.0);
            path.cubic_to(17.5, 18.0, 19.0, 16.0, 19.4, 14.0);
            line(&mut path, &[(19.5, 4.0), (19.5, 9.3), (14.2, 9.3)]);
            false
        }
        Icon::Home => {
            line(
                &mut path,
                &[
                    (4.0, 11.0),
                    (12.0, 4.0),
                    (20.0, 11.0),
                    (18.4, 11.0),
                    (18.4, 20.0),
                    (5.6, 20.0),
                    (5.6, 11.0),
                    (4.0, 11.0),
                ],
            );
            path.close();
            line(
                &mut path,
                &[(10.0, 20.0), (10.0, 14.5), (14.0, 14.5), (14.0, 20.0)],
            );
            false
        }
        Icon::Search => {
            path.push_circle(10.5, 10.5, 5.7);
            line(&mut path, &[(14.6, 14.6), (19.5, 19.5)]);
            false
        }
        Icon::Star | Icon::Starred => {
            line(
                &mut path,
                &[
                    (12.0, 3.5),
                    (14.7, 9.0),
                    (20.8, 9.9),
                    (16.4, 14.2),
                    (17.4, 20.2),
                    (12.0, 17.4),
                    (6.6, 20.2),
                    (7.6, 14.2),
                    (3.2, 9.9),
                    (9.3, 9.0),
                ],
            );
            path.close();
            matches!(icon, Icon::Starred)
        }
        Icon::Library => {
            rect(&mut path, 4.0, 5.0, 4.0, 14.0);
            rect(&mut path, 10.0, 5.0, 4.0, 14.0);
            line(
                &mut path,
                &[(16.0, 6.0), (18.9, 5.3), (22.0, 18.3), (19.1, 19.0)],
            );
            path.close();
            false
        }
        Icon::Download => {
            line(&mut path, &[(12.0, 4.0), (12.0, 14.0)]);
            line(&mut path, &[(7.5, 10.0), (12.0, 14.5), (16.5, 10.0)]);
            line(
                &mut path,
                &[(5.0, 15.5), (5.0, 19.5), (19.0, 19.5), (19.0, 15.5)],
            );
            false
        }
        Icon::Layout => {
            rect(&mut path, 3.5, 5.0, 17.0, 14.0);
            line(&mut path, &[(9.0, 5.0), (9.0, 19.0)]);
            false
        }
        Icon::Menu => {
            for y in [6.0, 12.0, 18.0] {
                path.push_circle(12.0, y, 1.35);
            }
            true
        }
        Icon::Plus => {
            line(&mut path, &[(12.0, 5.0), (12.0, 19.0)]);
            line(&mut path, &[(5.0, 12.0), (19.0, 12.0)]);
            false
        }
        Icon::Close => {
            line(&mut path, &[(6.5, 6.5), (17.5, 17.5)]);
            line(&mut path, &[(17.5, 6.5), (6.5, 17.5)]);
            false
        }
        Icon::Tab => {
            rect(&mut path, 4.0, 5.0, 16.0, 14.0);
            line(&mut path, &[(4.0, 9.0), (20.0, 9.0)]);
            false
        }
    };
    let path = path.finish()?;
    let [a, r, g, b] = color.to_be_bytes();
    let mut paint = Paint::default();
    paint.set_color_rgba8(r, g, b, a);
    let stroke = Stroke {
        width: 1.8,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    };
    for scale in [1, 2] {
        let size = 19 * scale;
        let mut bitmap = Pixmap::new(size, size)?;
        let transform = Transform::from_scale(size as f32 / 24.0, size as f32 / 24.0);
        if filled {
            bitmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
        } else {
            bitmap.stroke_path(&path, &paint, &stroke, transform, None);
        }
        if image.add_bitmap(
            scale as f32,
            size as i32,
            size as i32,
            ColorType::RGBA_8888,
            AlphaType::PREMULTIPLIED,
            Some(bitmap.data()),
        ) == 0
        {
            return None;
        }
    }
    Some(image)
}

fn line(path: &mut PathBuilder, points: &[(f32, f32)]) {
    if let Some(((x, y), rest)) = points.split_first() {
        path.move_to(*x, *y);
        for &(x, y) in rest {
            path.line_to(x, y);
        }
    }
}

fn rect(path: &mut PathBuilder, x: f32, y: f32, width: f32, height: f32) {
    if let Some(rect) = Rect::from_xywh(x, y, width, height) {
        path.push_rect(rect);
    }
}
