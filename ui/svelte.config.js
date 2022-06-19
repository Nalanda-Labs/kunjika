import netlify from '@sveltejs/adapter-netlify';
import vercel from '@sveltejs/adapter-vercel';
import adapter from '@sveltejs/adapter-node';

//export default {
//	kit: {
//		adapter: process.env.VERCEL ? vercel() : netlify()
//	}
//};

export default {
	kit: {
		adapter: adapter({ out: 'build' })
	}
}
