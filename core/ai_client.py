from __future__ import annotations

from typing import AsyncIterator, Optional

from openai import AsyncOpenAI

from .config import Settings


class AIClient:
    def __init__(self, settings: Settings) -> None:
        self._client = AsyncOpenAI(
            api_key=settings.ai_api_key,
            base_url=settings.ai_base_url,
        )
        self._model = settings.ai_model
        self._system_prompt = settings.ai_system_prompt

    async def chat(
        self,
        messages: list[dict[str, str]],
        *,
        stream: bool = False,
    ) -> str:
        full_messages = [{"role": "system", "content": self._system_prompt}, *messages]

        if stream:
            chunks: list[str] = []
            async for chunk in self._chat_stream(full_messages):
                chunks.append(chunk)
            return "".join(chunks)

        response = await self._client.chat.completions.create(
            model=self._model,
            messages=full_messages,
            stream=False,
        )
        return response.choices[0].message.content or ""

    async def chat_stream(
        self,
        messages: list[dict[str, str]],
    ) -> AsyncIterator[str]:
        full_messages = [{"role": "system", "content": self._system_prompt}, *messages]
        async for chunk in self._chat_stream(full_messages):
            yield chunk

    async def _chat_stream(
        self,
        messages: list[dict[str, str]],
    ) -> AsyncIterator[str]:
        stream = await self._client.chat.completions.create(
            model=self._model,
            messages=messages,
            stream=True,
        )
        async for chunk in stream:
            delta = chunk.choices[0].delta
            if delta and delta.content:
                yield delta.content
