export function filterWorkerLogLines(
  lines: string[],
  botId: string,
  sessionId?: string,
): string[] {
  if (botId.length === 0) {
    return lines;
  }

  const botMatches = lines.filter((line) => line.includes(botId));
  if (sessionId && sessionId.length > 0) {
    const sessionMatches = botMatches.filter((line) => line.includes(sessionId));
    if (sessionMatches.length > 0) {
      return sessionMatches;
    }
  }
  return botMatches;
}
