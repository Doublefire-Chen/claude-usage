import { useEffect, useState, useRef } from "react";

const SPARKLE_CHARS = ["·", "✻", "✽", "✶", "✳", "✢"];

const SPINNER_VERBS = [
  "Accomplishing", "Architecting", "Beboppin'", "Befuddling", "Bloviating",
  "Boondoggling", "Booping", "Brewing", "Canoodling", "Caramelizing",
  "Cascading", "Cerebrating", "Choreographing", "Churning", "Clauding",
  "Coalescing", "Cogitating", "Combobulating", "Contemplating", "Cooking",
  "Crafting", "Crunching", "Crystallizing", "Cultivating", "Deliberating",
  "Dilly-dallying", "Discombobulating", "Doodling", "Effecting", "Enchanting",
  "Envisioning", "Fermenting", "Fiddle-faddling", "Finagling", "Flamb\u00e9ing",
  "Flibbertigibbeting", "Flummoxing", "Forging", "Frolicking", "Gallivanting",
  "Garnishing", "Generating", "Gesticulating", "Gitifying", "Grooving",
  "Harmonizing", "Hatching", "Hullaballooing", "Hyperspacing", "Ideating",
  "Improvising", "Incubating", "Inferring", "Jitterbugging", "Kneading",
  "Levitating", "Lollygagging", "Manifesting", "Marinating", "Meandering",
  "Metamorphosing", "Moonwalking", "Moseying", "Mulling", "Musing",
  "Noodling", "Orchestrating", "Percolating", "Philosophising", "Pondering",
  "Pontificating", "Prestidigitating", "Processing", "Puzzling", "Quantumizing",
  "Razzle-dazzling", "Razzmatazzing", "Recombobulating", "Reticulating",
  "Ruminating", "Saut\u00e9ing", "Scampering", "Schlepping", "Shenaniganing",
  "Shimmying", "Simmering", "Skedaddling", "Sketching", "Smooshing",
  "Spelunking", "Spinning", "Sprouting", "Sublimating", "Swirling",
  "Synthesizing", "Thinking", "Tinkering", "Tomfoolering", "Topsy-turvying",
  "Transmuting", "Undulating", "Unfurling", "Vibing", "Waddling",
  "Wandering", "Whatchamacalliting", "Whirlpooling", "Whisking", "Wibbling",
  "Working", "Wrangling", "Zesting", "Zigzagging",
];

function formatTime(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return m > 0 ? `${m}m ${s}s` : `${s}s`;
}

function rand(min: number, max: number) {
  return Math.floor(Math.random() * (max - min)) + min;
}

export function ClaudeSparkle() {
  const [charIdx, setCharIdx] = useState(0);
  const [verb, setVerb] = useState(() => SPINNER_VERBS[rand(0, SPINNER_VERBS.length)]);
  const [elapsed, setElapsed] = useState(0);
  const [tokens, setTokens] = useState(0);
  const [phase, setPhase] = useState(0);
  const phaseRef = useRef(0);
  const phaseTimer = useRef<ReturnType<typeof setTimeout>>(undefined);
  const tokenTimer = useRef<ReturnType<typeof setTimeout>>(undefined);

  // Character cycling
  useEffect(() => {
    const id = setInterval(() => {
      setCharIdx((i) => (i + 1) % SPARKLE_CHARS.length);
    }, 120);
    return () => clearInterval(id);
  }, []);

  // Elapsed seconds
  useEffect(() => {
    const id = setInterval(() => {
      setElapsed((e) => e + 1);
    }, 1000);
    return () => clearInterval(id);
  }, []);

  // Random token increments
  useEffect(() => {
    function tick() {
      const inc = phaseRef.current === 0 ? rand(50, 300) : rand(200, 800);
      setTokens((t) => t + inc);
      tokenTimer.current = setTimeout(tick, rand(800, 2500));
    }
    tokenTimer.current = setTimeout(tick, rand(800, 2500));
    return () => clearTimeout(tokenTimer.current);
  }, []);

  // Phase transitions: ↑ → ↓ → new verb
  useEffect(() => {
    function tick() {
      const delay = phaseRef.current === 0 ? rand(2000, 5000) : rand(5000, 15000);
      phaseTimer.current = setTimeout(() => {
        if (phaseRef.current === 0) {
          phaseRef.current = 1;
          setPhase(1);
        } else {
          phaseRef.current = 0;
          setPhase(0);
          setVerb(SPINNER_VERBS[rand(0, SPINNER_VERBS.length)]);
          setElapsed(0);
          setTokens(0);
        }
        tick();
      }, delay);
    }
    tick();
    return () => clearTimeout(phaseTimer.current);
  }, []);

  const tokenStr = tokens >= 1000 ? `${(tokens / 1000).toFixed(1)}k` : `${tokens}`;

  return (
    <span className="claude-spinner">
      <span className="claude-sparkle">{SPARKLE_CHARS[charIdx]}</span>
      <span>{verb}… ({formatTime(elapsed)} · {phase === 0 ? "↑" : "↓"} {tokenStr} tokens)</span>
    </span>
  );
}
