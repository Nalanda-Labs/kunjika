use crate::state::{AppState, AppStateRaw};
use crate::utils::request::internal_error;

use ntex::web::{get, HttpResponse, Responder};

/// Questions per feed.
const FEED_ITEMS: i64 = 30;
/// The sitemap protocol allows at most 50,000 URLs per file.
const SITEMAP_URLS: i64 = 50_000;

/// Sent as `Content-Encoding` on the XML responses to opt them out of the
/// `Compress` middleware, which leaves a response alone when it already has
/// an encoding. With ntex 3.12 a large body (the sitemap is ~80 KB for a few
/// hundred questions) compressed for a client that accepts only gzip comes
/// out as a malformed chunked stream: curl reports "invalid chunk size" and
/// Node's fetch, which the frontend's `/sitemap.xml` relay uses, fails with
/// "Invalid character in chunk size". Browsers are unaffected (they also
/// accept brotli, and are then sent the body uncompressed).
const UNCOMPRESSED: &str = "identity";

/// Escape text for use inside an XML element or attribute.
fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            // Control characters other than tab/newline/CR are not allowed
            // in XML 1.0 at all, escaped or not.
            c if (c as u32) < 0x20 && !matches!(c, '\t' | '\n' | '\r') => {}
            c => out.push(c),
        }
    }
    out
}

fn rfc2822(unix: i64) -> String {
    chrono::DateTime::from_timestamp(unix, 0)
        .unwrap_or_default()
        .to_rfc2822()
}

fn rfc3339(unix: i64) -> String {
    chrono::DateTime::from_timestamp(unix, 0)
        .unwrap_or_default()
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn site(state: &AppStateRaw) -> String {
    format!("https://{}", state.config.host.trim_end_matches('/'))
}

struct FeedItem {
    id: i64,
    title: String,
    slug: String,
    excerpt: String,
    author: String,
    created: i64,
}

fn render_feed(state: &AppStateRaw, title: &str, page: &str, items: &[FeedItem]) -> HttpResponse {
    let site = site(state);
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\">\n<channel>\n");
    xml.push_str(&format!("<title>{}</title>\n", xml_escape(title)));
    xml.push_str(&format!("<link>{}</link>\n", xml_escape(&format!("{site}{page}"))));
    xml.push_str(&format!("<description>{}</description>\n", xml_escape(title)));
    if let Some(newest) = items.first() {
        xml.push_str(&format!("<lastBuildDate>{}</lastBuildDate>\n", rfc2822(newest.created)));
    }
    for i in items {
        let link = xml_escape(&format!("{site}/questions/{}/{}", i.id, i.slug));
        xml.push_str(&format!(
            "<item>\n<title>{}</title>\n<link>{link}</link>\n<guid isPermaLink=\"true\">{link}</guid>\n\
             <pubDate>{}</pubDate>\n<author>{}</author>\n<description>{}</description>\n</item>\n",
            xml_escape(&i.title),
            rfc2822(i.created),
            xml_escape(&i.author),
            xml_escape(&i.excerpt),
        ));
    }
    xml.push_str("</channel>\n</rss>\n");
    HttpResponse::Ok()
        .content_type("application/rss+xml; charset=utf-8")
        .header("Cache-Control", "public, max-age=300")
        .header("Content-Encoding", UNCOMPRESSED)
        .body(xml)
}

/// RSS feed of the newest questions.
#[get("/feeds/questions/")]
async fn questions_feed(state: AppState) -> impl Responder {
    let rows = sqlx::query!(
        r#"select p.id, p.title as "title!", p.slug, left(p.description, 500) as "excerpt!",
            p.created_at, u.username as "author?"
        from posts p left join users u on u.id = p.posted_by_id
        where coalesce(p.op_id, 0) = 0 and p.title is not null and p.title <> ''
        order by p.created_at desc, p.id desc
        limit $1"#,
        FEED_ITEMS
    )
    .fetch_all(&state.sql)
    .await;
    match rows {
        Ok(rows) => {
            let items: Vec<FeedItem> = rows
                .into_iter()
                .map(|r| FeedItem {
                    id: r.id,
                    title: r.title,
                    slug: r.slug.unwrap_or_default(),
                    excerpt: r.excerpt,
                    author: r.author.unwrap_or_default(),
                    created: r.created_at.unix_timestamp(),
                })
                .collect();
            render_feed(state.get_ref(), "Kunjika: newest questions", "/questions", &items)
        }
        Err(e) => internal_error(e),
    }
}

/// RSS feed of the newest questions in one tag. An unknown tag is an empty
/// feed rather than an error, so a reader subscribed to it keeps working.
#[get("/feeds/tagged/{tag}/")]
async fn tag_feed(params: ntex::web::types::Path<String>, state: AppState) -> impl Responder {
    let tag = params.into_inner();
    let rows = sqlx::query!(
        r#"select p.id, p.title as "title!", p.slug, left(p.description, 500) as "excerpt!",
            p.created_at, u.username as "author?"
        from posts p
        join post_tags pt on pt.post_id = p.id
        join tags t on t.id = pt.tag_id
        left join users u on u.id = p.posted_by_id
        where t.name = $1::text::citext
          and coalesce(p.op_id, 0) = 0 and p.title is not null and p.title <> ''
        order by p.created_at desc, p.id desc
        limit $2"#,
        tag,
        FEED_ITEMS
    )
    .fetch_all(&state.sql)
    .await;
    match rows {
        Ok(rows) => {
            let items: Vec<FeedItem> = rows
                .into_iter()
                .map(|r| FeedItem {
                    id: r.id,
                    title: r.title,
                    slug: r.slug.unwrap_or_default(),
                    excerpt: r.excerpt,
                    author: r.author.unwrap_or_default(),
                    created: r.created_at.unix_timestamp(),
                })
                .collect();
            render_feed(
                state.get_ref(),
                &format!("Kunjika: newest questions tagged {tag}"),
                &format!("/questions/tagged/{tag}"),
                &items,
            )
        }
        Err(e) => internal_error(e),
    }
}

/// Sitemap of every question, most recently active first. The frontend
/// serves this at `/sitemap.xml` on the site's own origin.
#[get("/sitemap.xml")]
async fn sitemap(state: AppState) -> impl Responder {
    let rows = sqlx::query!(
        r#"select id, slug, updated_at from posts
        where coalesce(op_id, 0) = 0 and title is not null and title <> ''
        order by updated_at desc, id desc
        limit $1"#,
        SITEMAP_URLS
    )
    .fetch_all(&state.sql)
    .await;
    match rows {
        Ok(rows) => {
            let site = site(state.get_ref());
            let mut xml = String::from(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
                 <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
            );
            for page in ["/", "/questions", "/tags", "/users"] {
                xml.push_str(&format!("<url><loc>{}</loc></url>\n", xml_escape(&format!("{site}{page}"))));
            }
            for r in rows {
                let loc = format!("{site}/questions/{}/{}", r.id, r.slug.unwrap_or_default());
                xml.push_str(&format!(
                    "<url><loc>{}</loc><lastmod>{}</lastmod></url>\n",
                    xml_escape(&loc),
                    rfc3339(r.updated_at.unix_timestamp())
                ));
            }
            xml.push_str("</urlset>\n");
            HttpResponse::Ok()
                .content_type("application/xml; charset=utf-8")
                .header("Cache-Control", "public, max-age=3600")
                .header("Content-Encoding", UNCOMPRESSED)
                .body(xml)
        }
        Err(e) => internal_error(e),
    }
}

/// robots.txt, pointing crawlers at the sitemap. Generated here because the
/// Sitemap line needs the site's public host, which only the backend config
/// knows; the frontend serves it at `/robots.txt`.
#[get("/robots.txt")]
async fn robots(state: AppState) -> impl Responder {
    HttpResponse::Ok().content_type("text/plain; charset=utf-8").body(format!(
        "# https://www.robotstxt.org/robotstxt.html\nUser-agent: *\nDisallow:\n\nSitemap: {}/sitemap.xml\n",
        site(state.get_ref())
    ))
}

pub fn init(cfg: &mut ntex::web::ServiceConfig) {
    cfg.service(robots);
    cfg.service(questions_feed);
    cfg.service(tag_feed);
    cfg.service(sitemap);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_escape_covers_markup_and_drops_control_characters() {
        assert_eq!(
            xml_escape("a < b && \"c\" > 'd'\u{0001}\n"),
            "a &lt; b &amp;&amp; &quot;c&quot; &gt; &apos;d&apos;\n"
        );
    }

    #[test]
    fn dates_are_formatted_for_rss_and_sitemaps() {
        assert_eq!(rfc2822(0), "Thu, 1 Jan 1970 00:00:00 +0000");
        assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
    }
}
