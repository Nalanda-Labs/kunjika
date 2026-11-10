/** The user snapshot stored in the `logged_in` cookie. Display only: never used for authorization. */
export interface SessionUser {
    id: number;
    username: string;
    email: string;
    image_url: string;
    created_date?: string;
    location?: string | null;
    designation?: string | null;
    git?: string | null;
    website?: string | null;
    is_superuser?: boolean | null;
    is_moderator?: boolean | null;
}

export function isReviewer(user: SessionUser | null | undefined): boolean {
    return !!user && !!(user.is_moderator || user.is_superuser);
}

/** A question row in the lists (`QR` in the backend). Tags arrive comma-joined. */
export interface QuestionRow {
    id: string;
    title: string;
    slug: string;
    votes: number;
    views: number;
    answers: number;
    answer_accepted: boolean;
    username: string;
    uid: string;
    image_url: string;
    tags: string;
    tid: string;
    cat: number;
    uat: number;
}

export interface QuestionsPayload {
    questions: QuestionRow[];
    count: number;
}

/** A question as returned by `GET questions/{id}/{slug}`. */
export interface QuestionDetail {
    id: string;
    title: string;
    description: string;
    votes: number;
    views: number;
    posted_by_id: string;
    username: string;
    image_url: string | null;
    tags: string[];
    vote_by_current_user: number;
    cat: number;
    uat: number;
    /** A closed question is readable but takes no new answers. */
    closed: boolean;
    /** Set when the question was closed as a duplicate of another one. */
    duplicate_of: { id: string; title: string; slug: string } | null;
    /** Whether the post has earlier versions to show under "History". */
    edited: boolean;
    /** Whether the signed-in reader has muted this thread. */
    muted: boolean;
}

/** An answer on a question page (`AR` in the backend). */
export interface AnswerRow {
    question_id: string;
    description: string;
    votes: number;
    posted_by_id: string;
    username: string;
    image_url: string;
    answer_accepted: boolean;
    reply_to_id: number | null;
    rusername: string | null;
    rimage_url: string | null;
    vote_by_current_user: number;
    created_at: string;
    cat: number;
    edited: boolean;
}

/** An answer or bookmark in a user's profile tabs (`AR2` in the backend). */
export interface UserPostRow {
    question_id: number;
    answer_id: number;
    title: string;
    slug: string;
    votes: number;
    answer_accepted: boolean;
    tags: string;
    tid: string;
    uat: number;
}

/** A post as returned by `GET edit/question/{id}` (question or answer). */
export interface EditablePost {
    title: string;
    description: string;
    tags: string[];
    created_at: string;
    votes: number;
    posted_by_id: number;
    username: string;
    image_url: string | null;
}

export interface Tag {
    id: number;
    name: string;
    info: string | null;
    post_count: number | null;
    daily_count: number | null;
    weekly_count: number | null;
}

export interface UserCard {
    id: number;
    username: string;
    displayname: string;
    name: string;
    location: string;
    image_url: string;
    karma: number;
}

export interface Profile {
    id: number;
    username: string;
    displayname: string;
    designation: string;
    location: string;
    image_url: string;
    git: string;
    website: string;
    karma: number;
    created_date: string;
    cat: number;
    /** `normal` or `suspended`. */
    status: string;
}

export interface Report {
    id: number;
    post_id: number;
    reason: string;
    details: string;
    status: string;
    created_at: string;
    reported_by_id: number;
    reporter: string;
    author_id: number | null;
    author: string | null;
    post_title: string | null;
    post_excerpt: string | null;
    is_question: boolean | null;
    open_count: number | null;
}

export type NotificationKind = 'answer' | 'reply' | 'accepted' | 'upvote' | 'tag_question';

export interface Notification {
    id: number;
    kind: NotificationKind;
    actor_id: number | null;
    actor: string | null;
    question_id: number | null;
    question_title: string | null;
    question_slug: string | null;
    post_id: number | null;
    cat: number;
    read: boolean;
}

/** One earlier version of a post (`posts/{id}/history`). */
export interface Revision {
    id: number;
    title: string | null;
    description: string;
    tags: string | null;
    edited_by_id: number | null;
    editor: string | null;
    /** When this version was replaced by the next one. */
    cat: number;
}

export interface PostHistory {
    question_id: number;
    question_slug: string;
    is_question: boolean;
    title: string | null;
    description: string;
    author: string | null;
    posted_by_id: number;
    revisions: Revision[];
}

/** One change to a user's karma (`users/{id}/karma/`). Never says who voted. */
export interface KarmaEvent {
    id: number;
    delta: number;
    reason: 'vote_up' | 'vote_down' | 'accepted' | 'unaccepted' | 'post_deleted' | 'opening_balance';
    post_id: number | null;
    /** Null once the post has been deleted. */
    question_id: number | null;
    question_title: string | null;
    question_slug: string | null;
    is_answer: boolean;
    cat: number;
}

export interface ModLogEntry {
    id: number;
    action: 'delete_post' | 'close_question' | 'reopen_question' | 'suspend_user' | 'reinstate_user';
    actor_id: number;
    actor: string | null;
    target_user_id: number | null;
    target_user: string | null;
    post_id: number | null;
    /** Null once the post has been deleted. */
    question_id: number | null;
    question_slug: string | null;
    details: string;
    cat: number;
}
