// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {
	namespace App {
		interface Locals {
			user: {
				id: bigint,
				email: string,
				username: string,
				firstName: string,
				lastName: string,
				isAdmin: boolean,
				image_url: string
			} | null
		}
		interface PageData {
			user: {
				id: bigint,
				email: string,
				username: string,
				firstName: string,
				lastName: string,
				isAdmin: boolean,
				image_url: string
			} | null
		}
	}
}

export {};
