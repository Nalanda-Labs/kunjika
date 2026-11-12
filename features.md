# Kunjika features

Kunjika is a lean question-and-answer forum: a Rust (ntex) API, a SvelteKit
frontend, YugabyteDB for storage and Redis for sessions and rate limits. This
file lists what the application does today. For installation see
[`README.md`](README.md); for the test suites see [`TESTING.md`](TESTING.md).

## Accounts and sign-in

- **Registration** with a username (3-33 characters) and an email address,
  with a live check that the username is free.
- **Email confirmation**: a confirmation link is mailed on registration and
  can be resent when it has expired.
- **Passwordless sign-in**: the user enters their email and receives a
  one-time code, valid for 5 minutes. There are no passwords to store or leak.
- **Sessions** use RS256-signed JWT access and refresh tokens held in
  `HttpOnly` cookies and tracked in Redis. The frontend refreshes the session
  in the background, and refresh rotates both tokens.
- **Sign-out** revokes the session's tokens.
- **Account deletion**: a user can delete their own account. The username and
  email are anonymised and the account can no longer sign in.

## Profiles

- Public profile page with username, designation, location, git and website
  links, and karma.
- Profile picture: Gravatar by default, or an uploaded image.
- Profile tabs: summary, questions asked, answers given, bookmarks (visible
  only to the owner) and karma history.
- User directory with paging and search by username.

## Questions and answers

- **Ask a question** with a title (6-256 characters), a Markdown body
  (20-100,000 characters) and up to 5 tags.
- **Similar questions** are suggested while the title is typed, so an
  existing answer can be found before a duplicate is posted.
- **Answer** a question, or **reply** to a specific answer; the reply shows
  whom it is addressed to.
- Answers are listed in **chronological order**, 20 to a page, Discourse
  style. They are not re-sorted by votes, and there are no comments.
- A link to a single answer opens the page that answer is on and highlights
  it. "Share" copies that link.
- **Accept an answer**: the asker can mark one answer as accepted, and can
  undo it.
- **Edit** your own questions and answers. Every edit is kept, and the
  **edit history** of a post is public, with a diff between versions.
- **Delete** your own answers, and your own questions while they have no
  answers.
- **Bookmark** answers for later.
- Question lists: all questions (most recently active first, with pinned
  questions on top), unanswered questions, questions by tag, and questions by
  user.
- Readable question URLs of the form `/questions/{id}/{slug}`.

## Writing and formatting

- Markdown editor with a toolbar, a live preview and keyboard shortcuts
  (Ctrl/Cmd + B, I, K).
- GitHub-flavoured Markdown, rendered with `marked` and sanitised with
  DOMPurify.
- Syntax highlighting for code blocks (highlight.js), each labelled with its
  language.
- TeX maths (MathJax), including `$…$` inline maths.
- Note, tip and warning blocks.
- **Image upload** from the editor. Only PNG, JPEG, GIF and WebP are
  accepted, checked by file content rather than by name, with a configurable
  size limit (2 MB in the sample config).
- A built-in Markdown help page.

## Tags

- Up to 5 tags per question, with autocomplete while typing.
- Tag directory with paging and search.
- A tag information page for each tag; moderators can edit it.
- **Follow a tag** to be notified of new questions in it.
- An RSS feed for each tag.

## Voting and karma

- Vote questions and answers up or down: one vote per user per post, and not
  on your own posts.
- **Karma**: the author gains karma for a vote up and loses it for a vote
  down (10 each way in the sample config), and gains more when their answer
  is accepted (20). Karma never drops below 1. All three amounts are
  configurable.
- **Karma history**: a public, itemised list of every change to a user's
  karma, showing the reason and the post. It never shows who voted.

## Notifications

- In-app notifications, with an unread count on the bell in the navigation
  bar, for:
  - a new answer to your question,
  - a reply to your answer,
  - your answer being accepted,
  - a vote up on your post,
  - a new question in a tag you follow.
- The bell is updated every 30 seconds while the tab is visible.
- Mark one notification, or all of them, as read.
- **Email notifications** for new answers, which each user can switch off.
- **Mute a question** to stop notifications and emails about new answers in
  it.

## Search and discovery

- **Full-text search** over questions and their answers, ranked by relevance
  (PostgreSQL-style `tsvector` search with GIN indexes).
- Search for tags and users by name.
- **RSS feeds** of the newest questions, site-wide and per tag.
- `sitemap.xml` (up to 50,000 questions) and `robots.txt`, generated from
  the live data.

## Moderation

Roles are user, moderator and superuser.

- **Reports**: any signed-in user can report a question or answer as spam,
  abusive, off-topic, low-quality, duplicate or other, with optional details.
  A user cannot report their own post, cannot hold two open reports on the
  same post, and can file at most 5 reports in 24 hours.
- **Report queue** for moderators, with an open-report count in the sidebar.
  A report is marked reviewed, dismissed or actioned; "actioned" deletes the
  reported post.
- **Close and reopen** questions. A closed question stays readable but takes
  no new answers, and it can be closed as a duplicate of another question.
- **Delete any post**. Deleting a question deletes its answers too.
- **Pin** questions to the top of the front page (at most 5 at once).
- **Suspend and reinstate** users. A suspension takes effect at once, even
  for sessions that are already signed in. Moderators cannot suspend other
  moderators; a superuser can. Nobody can suspend a superuser.
- **Moderation log**, visible to moderators: who deleted, closed, reopened,
  suspended or reinstated what, and when.

## Security and abuse prevention

- **XSRF protection**: every authenticated request must carry an
  `X-XSRF-Token` header that matches the session.
- **Rate limits**, backed by Redis and configurable per hour: questions (10),
  answers (30), registrations per IP address (5), sign-in codes per email
  address (5) and sign-in attempts (10). A limit of 0 switches that check
  off. If Redis is unreachable the limiter lets requests through.
- All input is validated on the server, and SQL queries are checked against
  the schema at compile time (`sqlx`).
- Rendered Markdown is sanitised before display.
- Uploads are restricted to image formats that cannot carry script.
- The backend refuses to run as root unless explicitly allowed (for
  containers).

## Operations

- A single Rust binary, with one worker per CPU core.
- JSON configuration file, with environment overrides for containers.
- **Database migrations** are applied automatically at startup (can be
  switched off), or by hand with the scripts in `backend/migrations`.
- Runs on **YugabyteDB** only; plain PostgreSQL is not supported.
- A compose file (`podman-test-compose.yaml`) brings up the database, Redis,
  a mail catcher, the backend and the frontend with one command.
- Sample `systemd` unit and nginx configuration in `backend/`.
- Keyset (cursor) pagination on notifications, karma history, the report
  queue and the moderation log, so pages stay correct while rows are added.
- Test suites: backend REST tests, backend unit tests and Playwright
  end-to-end tests, run in CI.

## Not included, by design

- No comments on posts: replies are answers.
- No sorting of answers by votes.
- No password sign-in and no third-party (OAuth) sign-in.
- No integration with closed-source or non-GPL applications.

## Licence

GNU GPL v3.
