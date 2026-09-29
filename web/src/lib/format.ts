export const STATUS_LABEL: Record<string, string> = {
	PENDING: '기다리는 중',
	JUDGING: '채점 중',
	AC: '맞았습니다!!',
	WA: '틀렸습니다',
	TLE: '시간 초과',
	MLE: '메모리 초과',
	RE: '런타임 에러',
	CE: '컴파일 에러',
	SE: '채점 오류'
};

export const STATUS_ORDER = ['AC', 'WA', 'TLE', 'MLE', 'RE', 'CE', 'SE', 'PENDING', 'JUDGING'];

export const LANGUAGE_LABEL: Record<string, string> = {
	c: 'C',
	cpp17: 'C++17',
	python3: 'Python 3',
	java: 'Java',
	rust: 'Rust'
};

export const KIND_LABEL: Record<string, string> = {
	QUESTION: '질문',
	COUNTEREXAMPLE: '반례',
	TYPO: '오타 제보'
};

export function isPending(status: string) {
	return status === 'PENDING' || status === 'JUDGING';
}

export function dateTime(iso: string | null | undefined): string {
	if (!iso) return '';
	const d = new Date(iso);
	const p = (n: number) => String(n).padStart(2, '0');
	return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}

export function date(iso: string | null | undefined): string {
	return dateTime(iso).slice(0, 10);
}

export function bytes(n: number): string {
	if (n < 1024) return `${n} B`;
	if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
	return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

export function num(n: number): string {
	return n.toLocaleString('en-US');
}

/** Rows with PENDING/JUDGING need polling. */
export function anyPending(rows: { status: string }[]): boolean {
	return rows.some((r) => isPending(r.status));
}
