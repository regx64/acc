import { api } from '$lib/server/api';
import type { Actions } from './$types';

export const actions: Actions = {
	default: async (event) => {
		const f = await event.request.formData();
		await api(event).POST('/auth/password-reset/request', { body: { email: String(f.get('email') ?? '') } });
		return { ok: '가입된 주소라면 재설정 링크를 보냈습니다. 링크는 30분 동안 한 번만 쓸 수 있습니다.' };
	}
};
