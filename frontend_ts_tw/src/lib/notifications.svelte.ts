import * as api from './api.ts';

/** Unread notification count, shared by the bell in the nav and the notifications page. */
export const unread = $state({ count: 0 });

/** Refresh the unread count. Best effort: the bell just keeps its last value on failure. */
export async function refreshUnread(): Promise<void> {
    try {
        const res = await api.post('notifications/unread-count/', {});
        if (res.status === 200) unread.count = (await api.json<{ count?: number }>(res)).count ?? 0;
    } catch {
        // offline or the backend is restarting; try again on the next navigation
    }
}
