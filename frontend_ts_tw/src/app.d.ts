import type { SessionUser } from '#lib/types.ts';

declare global {
    namespace App {
        interface Locals {
            user: SessionUser | null;
        }
        interface PageData {
            user: SessionUser | null;
        }
    }
}

export {};
