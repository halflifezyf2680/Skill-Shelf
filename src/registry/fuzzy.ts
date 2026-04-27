type RankedMatch = {
  score: number;
  reasons: string[];
};

export function scoreSkillText(fields: {
  id: string;
  name: string;
  group: string;
  summary: string;
  tags: string[];
  query: string;
}): RankedMatch {
  const normalizedQuery = normalize(fields.query);
  const queryTokens = tokenize(normalizedQuery);
  if (normalizedQuery.length === 0) {
    return { score: 0, reasons: [] };
  }

  let score = 0;
  const reasons: string[] = [];

  const idNorm = normalize(fields.id);
  const nameNorm = normalize(fields.name);
  const groupNorm = normalize(fields.group);
  const summaryNorm = normalize(fields.summary);
  const tagNorms = fields.tags.map(normalize);

  if (idNorm === normalizedQuery || nameNorm === normalizedQuery) {
    score += 120;
    reasons.push("exact-name");
  }

  if (startsOrEndsWith(idNorm, normalizedQuery) || startsOrEndsWith(nameNorm, normalizedQuery)) {
    score += 65;
    reasons.push("prefix-suffix");
  }

  if (
    idNorm.includes(normalizedQuery) ||
    nameNorm.includes(normalizedQuery) ||
    summaryNorm.includes(normalizedQuery)
  ) {
    score += 40;
    reasons.push("substring");
  }

  const fieldTokens = new Set([
    ...tokenize(idNorm),
    ...tokenize(nameNorm),
    ...tokenize(groupNorm),
    ...tokenize(summaryNorm),
    ...tagNorms.flatMap(tokenize),
  ]);
  const tokenHits = queryTokens.filter((token) => fieldTokens.has(token));
  if (tokenHits.length > 0) {
    score += tokenHits.length * 18;
    reasons.push(`token-overlap:${tokenHits.length}`);
  }

  const bestLev = Math.min(
    normalizedLevenshteinScore(normalizedQuery, idNorm),
    normalizedLevenshteinScore(normalizedQuery, nameNorm),
    ...tagNorms.map((tag) => normalizedLevenshteinScore(normalizedQuery, tag)),
  );
  if (bestLev <= 0.34) {
    score += Math.round((1 - bestLev) * 30);
    reasons.push(`levenshtein:${bestLev.toFixed(2)}`);
  }

  if (groupNorm.includes(normalizedQuery) || queryTokens.some((token) => groupNorm.includes(token))) {
    score += 10;
    reasons.push("group");
  }

  return { score, reasons: Array.from(new Set(reasons)) };
}

function startsOrEndsWith(value: string, query: string): boolean {
  return value.startsWith(query) || value.endsWith(query);
}

export function normalize(value: string): string {
  return value.toLowerCase().replace(/[_/\\-]+/g, " ").replace(/\s+/g, " ").trim();
}

function tokenize(value: string): string[] {
  const tokens: string[] = [];
  const segments = value.split(/[^a-z0-9\u4e00-\u9fff]+/u);
  for (const segment of segments) {
    if (!segment) continue;
    // Latin/number tokens
    if (/^[a-z0-9]+$/u.test(segment)) {
      if (segment.length >= 2) tokens.push(segment);
      continue;
    }
    // CJK sequence: generate overlapping bigrams
    const cjk = segment.replace(/[^\u4e00-\u9fff]/gu, "");
    for (let i = 0; i < cjk.length - 1; i++) {
      tokens.push(cjk.substring(i, i + 2));
    }
    // Keep full CJK segment as a token for exact phrase match
    if (cjk.length >= 2) tokens.push(cjk);
  }
  return tokens;
}

export function normalizedLevenshteinScore(a: string, b: string): number {
  if (!a || !b) {
    return 1;
  }
  const distance = levenshtein(a, b);
  return distance / Math.max(a.length, b.length);
}

function levenshtein(a: string, b: string): number {
  if (a === b) {
    return 0;
  }
  if (a.length === 0) {
    return b.length;
  }
  if (b.length === 0) {
    return a.length;
  }

  const dp = Array.from({ length: a.length + 1 }, () => new Array<number>(b.length + 1).fill(0));
  for (let i = 0; i <= a.length; i += 1) {
    dp[i][0] = i;
  }
  for (let j = 0; j <= b.length; j += 1) {
    dp[0][j] = j;
  }

  for (let i = 1; i <= a.length; i += 1) {
    for (let j = 1; j <= b.length; j += 1) {
      const cost = a[i - 1] === b[j - 1] ? 0 : 1;
      dp[i][j] = Math.min(dp[i - 1][j] + 1, dp[i][j - 1] + 1, dp[i - 1][j - 1] + cost);
    }
  }

  return dp[a.length][b.length];
}
