import * as api from './api.ts';
import type { QuestionsPage } from './components/PagedQuestions.svelte';
import type { QuestionRow } from './types.ts';

interface ListResponse {
    data?: { questions: QuestionRow[]; count?: number };
    pinned?: { questions: QuestionRow[] };
    count?: number;
    status?: string | boolean;
    message?: string;
}

/**
 * POST a question-list endpoint and normalise the answer. The backend
 * reports an empty list as an error carrying `api.NO_ROWS`; that becomes an
 * empty page. Any other failure throws with the server's message.
 */
export async function fetchQuestions(path: string, body: Record<string, unknown>): Promise<QuestionsPage> {
    const res = await api.post(path, body);
    const j = await api.json<ListResponse>(res);
    if (res.status !== 200) {
        if (j.message === api.NO_ROWS) return { questions: [], count: 0 };
        throw new Error(j.message || 'Could not load questions.');
    }
    return {
        questions: j.data?.questions ?? [],
        count: j.count ?? j.data?.count ?? 0,
        pinned: j.pinned?.questions
    };
}
