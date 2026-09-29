// Level groups ℕ ℤ ℚ ℝ ℂ ℍ × 5 steps. Mirrors acc_core::level.

export const TIER_COEFF = 90;

export type Group = { key: string; symbol: string; name: string; ko: string };

export const UNRATED: Group = { key: 'unrated', symbol: '?', name: 'Unrated', ko: '미정' };
export const GROUPS: Group[] = [
	{ key: 'natural', symbol: 'ℕ', name: 'Natural', ko: '자연수' },
	{ key: 'integer', symbol: 'ℤ', name: 'Integer', ko: '정수' },
	{ key: 'rational', symbol: 'ℚ', name: 'Rational', ko: '유리수' },
	{ key: 'real', symbol: 'ℝ', name: 'Real', ko: '실수' },
	{ key: 'complex', symbol: 'ℂ', name: 'Complex', ko: '복소수' },
	{ key: 'quaternion', symbol: 'ℍ', name: 'Quaternion', ko: '사원수' }
];

export function groupIndex(level: number): number {
	return level <= 0 || level > 30 ? 0 : Math.ceil(level / 5);
}

export function group(level: number): Group {
	const i = groupIndex(level);
	return i === 0 ? UNRATED : GROUPS[i - 1];
}

export function step(level: number): number {
	return level <= 0 || level > 30 ? 0 : ((level - 1) % 5) + 1;
}

export function label(level: number): string {
	return level <= 0 ? 'Unrated' : `${group(level).symbol}${step(level)}`;
}

export function longLabel(level: number): string {
	return level <= 0 ? 'Unrated' : `${group(level).name} ${step(level)}`;
}

export function tierThreshold(tier: number): number {
	if (tier <= 0) return 0;
	if (tier === 1) return 1;
	return TIER_COEFF * tier * tier;
}
