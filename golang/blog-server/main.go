package main

import (
	"flag"
	"fmt"
	"html/template"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"github.com/gomarkdown/markdown"
	"github.com/gomarkdown/markdown/html"
	"github.com/gomarkdown/markdown/parser"
)

type Article struct {
	Slug      string
	Title     string
	BodyHTML  template.HTML
	UpdatedAt time.Time
}

type Blog struct {
	ArticlesDir string
	articles    []Article
}

func main() {
	port := flag.Int("port", 8080, "HTTP port to listen on")
	dir := flag.String("dir", "articles", "directory containing markdown articles")
	flag.Parse()

	blog := &Blog{ArticlesDir: *dir}
	if err := blog.loadArticles(); err != nil {
		log.Fatalf("failed to load articles: %v", err)
	}

	mux := http.NewServeMux()
	mux.HandleFunc("/", blog.handleIndex)
	mux.HandleFunc("/article/", blog.handleArticle)
	mux.HandleFunc("/style.css", handleCSS)

	addr := fmt.Sprintf(":%d", *port)
	log.Printf("Blog server listening on http://localhost%s", addr)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Fatalf("server error: %v", err)
	}
}

func (b *Blog) loadArticles() error {
	entries, err := os.ReadDir(b.ArticlesDir)
	if err != nil {
		return fmt.Errorf("read articles dir: %w", err)
	}

	var articles []Article
	for _, entry := range entries {
		if entry.IsDir() || filepath.Ext(entry.Name()) != ".md" {
			continue
		}

		path := filepath.Join(b.ArticlesDir, entry.Name())
		info, err := entry.Info()
		if err != nil {
			continue
		}

		raw, err := os.ReadFile(path)
		if err != nil {
			continue
		}

		slug := strings.TrimSuffix(entry.Name(), ".md")
		title := extractTitle(string(raw))
		html := renderMarkdown(raw)

		articles = append(articles, Article{
			Slug:      slug,
			Title:     title,
			BodyHTML:  template.HTML(html),
			UpdatedAt: info.ModTime(),
		})
	}

	sort.Slice(articles, func(i, j int) bool {
		return articles[i].UpdatedAt.After(articles[j].UpdatedAt)
	})

	b.articles = articles
	return nil
}

func extractTitle(raw string) string {
	lines := strings.SplitN(raw, "\n", 3)
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		if strings.HasPrefix(trimmed, "# ") {
			return strings.TrimPrefix(trimmed, "# ")
		}
	}
	return "Untitled"
}

func renderMarkdown(input []byte) string {
	extensions := parser.CommonExtensions | parser.AutoHeadingIDs | parser.NoEmptyLineBeforeBlock
	p := parser.NewWithExtensions(extensions)

	htmlFlags := html.CommonFlags | html.HrefTargetBlank
	opts := html.RendererOptions{Flags: htmlFlags}
	renderer := html.NewRenderer(opts)

	return string(markdown.ToHTML(input, p, renderer))
}

func (b *Blog) handleIndex(w http.ResponseWriter, r *http.Request) {
	if r.URL.Path != "/" {
		http.NotFound(w, r)
		return
	}

	var sb strings.Builder
	sb.WriteString(pageHead("My Blog"))
	sb.WriteString(`<div class="container"><h1>My Blog</h1><ul class="article-list">`)

	for _, a := range b.articles {
		sb.WriteString(fmt.Sprintf(
			`<li><a href="/article/%s">%s</a><span class="date">%s</span></li>`,
			template.HTMLEscapeString(a.Slug),
			template.HTMLEscapeString(a.Title),
			a.UpdatedAt.Format("2006-01-02"),
		))
	}

	sb.WriteString(`</ul></div>`)
	sb.WriteString(pageFoot())
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	w.Write([]byte(sb.String()))
}

func (b *Blog) handleArticle(w http.ResponseWriter, r *http.Request) {
	slug := strings.TrimPrefix(r.URL.Path, "/article/")
	slug = strings.TrimSuffix(slug, "/")

	var found *Article
	for _, a := range b.articles {
		if a.Slug == slug {
			found = &a
			break
		}
	}

	if found == nil {
		http.NotFound(w, r)
		return
	}

	var sb strings.Builder
	sb.WriteString(pageHead(found.Title))
	sb.WriteString(`<div class="container"><article>`)
	sb.WriteString(fmt.Sprintf(
		`<p class="back-link"><a href="/">&larr; Back to home</a></p>
		<h1>%s</h1>
		<p class="article-date">%s</p>
		<div class="article-body">%s</div>`,
		template.HTMLEscapeString(found.Title),
		found.UpdatedAt.Format("January 2, 2006"),
		string(found.BodyHTML),
	))
	sb.WriteString(`</article></div>`)
	sb.WriteString(pageFoot())
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	w.Write([]byte(sb.String()))
}

func handleCSS(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "text/css; charset=utf-8")
	w.Write([]byte(css))
}

func pageHead(title string) string {
	return fmt.Sprintf(`<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>%s</title>
<link rel="stylesheet" href="/style.css">
</head>
<body>`, template.HTMLEscapeString(title))
}

func pageFoot() string {
	return `</body></html>`
}

const css = `
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  line-height: 1.7;
  color: #1a1a1a;
  background: #fafafa;
}
.container {
  max-width: 720px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
}
h1 { font-size: 2rem; margin-bottom: 0.5rem; color: #111; }
h2 { font-size: 1.4rem; margin: 2rem 0 0.75rem; color: #222; }
h3 { font-size: 1.15rem; margin: 1.5rem 0 0.5rem; color: #333; }
p { margin-bottom: 1rem; }
a { color: #2563eb; text-decoration: none; }
a:hover { text-decoration: underline; }
.article-list { list-style: none; margin-top: 1.5rem; }
.article-list li {
  display: flex; justify-content: space-between; align-items: baseline;
  padding: 0.75rem 0; border-bottom: 1px solid #e5e5e5;
}
.article-list li a { font-size: 1.1rem; font-weight: 500; }
.date { color: #666; font-size: 0.9rem; white-space: nowrap; margin-left: 1rem; }
.back-link { margin-bottom: 1.5rem; }
.article-date { color: #666; font-size: 0.95rem; margin-bottom: 2rem; }
.article-body { font-size: 1.05rem; }
.article-body ul, .article-body ol { margin: 0 0 1rem 1.5rem; }
.article-body li { margin-bottom: 0.3rem; }
.article-body blockquote {
  border-left: 4px solid #2563eb; margin: 1.5rem 0; padding: 0.5rem 1rem;
  background: #f0f4ff; color: #333; font-style: italic;
}
.article-body pre {
  background: #1e1e1e; color: #d4d4d4; padding: 1rem 1.25rem;
  border-radius: 6px; overflow-x: auto; margin: 1rem 0; font-size: 0.9rem; line-height: 1.5;
}
.article-body code {
  font-family: "SF Mono", "Fira Code", "Cascadia Code", Consolas, monospace;
  font-size: 0.9em;
}
.article-body :not(pre) > code {
  background: #eee; color: #c7254e; padding: 0.15em 0.4em; border-radius: 3px;
}
.article-body img { max-width: 100%; height: auto; border-radius: 4px; }
.article-body hr { border: none; border-top: 1px solid #ddd; margin: 2rem 0; }
`
