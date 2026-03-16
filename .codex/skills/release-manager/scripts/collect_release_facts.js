#!/usr/bin/env node

const { execFileSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

function parseArgs(argv) {
  const options = {
    repo: ".",
    fromRef: undefined,
    toRef: "HEAD",
    limit: 50,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--repo") {
      options.repo = argv[++i];
    } else if (arg === "--from-ref") {
      options.fromRef = argv[++i];
    } else if (arg === "--to-ref") {
      options.toRef = argv[++i];
    } else if (arg === "--limit") {
      options.limit = Number.parseInt(argv[++i], 10);
    } else if (arg === "--help" || arg === "-h") {
      printHelp();
      process.exit(0);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }

  if (!Number.isInteger(options.limit) || options.limit < 0) {
    throw new Error("--limit must be a non-negative integer");
  }

  return options;
}

function printHelp() {
  process.stdout.write(
    [
      "Collect release facts from git history for changelog and release drafting.",
      "",
      "Usage:",
      "  node collect_release_facts.js [--repo PATH] [--from-ref REF] [--to-ref REF] [--limit N]",
      "",
      "Options:",
      "  --repo PATH      Repository root path (default: .)",
      "  --from-ref REF   Start ref. Defaults to latest v* tag or root commit.",
      "  --to-ref REF     End ref (default: HEAD)",
      "  --limit N        Maximum number of commits to print (default: 50)",
      "",
    ].join("\n"),
  );
}

function git(repo, args, { check = true } = {}) {
  try {
    return execFileSync("git", ["-C", repo, ...args], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  } catch (error) {
    if (!check) {
      const stdout = error.stdout ? String(error.stdout).trim() : "";
      return stdout;
    }
    const stderr = error.stderr ? String(error.stderr).trim() : "";
    throw new Error(stderr || "git command failed");
  }
}

function latestTag(repo) {
  const output = git(repo, ["tag", "--list", "v*", "--sort=-version:refname"], {
    check: false,
  });
  const tags = output.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
  return tags[0];
}

function revParse(repo, ref) {
  return git(repo, ["rev-parse", ref]);
}

function mergeBase(repo, a, b) {
  return git(repo, ["merge-base", a, b]);
}

function rootCommit(repo) {
  return git(repo, ["rev-list", "--max-parents=0", "HEAD"])
    .split(/\r?\n/)[0]
    .trim();
}

function logRange(repo, start, end, limit) {
  const range = start ? `${start}..${end}` : end;
  const output = git(repo, ["log", "--format=%H%x09%s", range], { check: false });
  return output
    .split(/\r?\n/)
    .filter(Boolean)
    .slice(0, limit)
    .map((line) => {
      const [sha, subject] = line.split("\t", 2);
      return { sha, subject };
    });
}

function changedFiles(repo, start, end) {
  const range = start ? `${start}..${end}` : end;
  const output = git(repo, ["diff", "--name-only", range], { check: false });
  return output.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
}

function topLevelAreas(paths) {
  const counts = new Map();
  for (const filePath of paths) {
    const area = filePath.includes("/") ? filePath.split("/", 1)[0] : "(repo-root)";
    counts.set(area, (counts.get(area) || 0) + 1);
  }
  return [...counts.entries()].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
}

function main() {
  const options = parseArgs(process.argv.slice(2));
  const repo = path.resolve(options.repo);

  if (!fs.existsSync(path.join(repo, ".git"))) {
    process.stderr.write(`error: ${repo} does not look like a git repository\n`);
    process.exit(1);
  }

  const end = revParse(repo, options.toRef);
  const latest = latestTag(repo);

  let start;
  let startLabel;
  if (options.fromRef) {
    start = revParse(repo, options.fromRef);
    startLabel = options.fromRef;
  } else if (latest) {
    start = revParse(repo, latest);
    startLabel = latest;
  } else {
    start = rootCommit(repo);
    startLabel = `${start.slice(0, 12)} (root commit)`;
  }

  const base = mergeBase(repo, start, end);
  const compareStart = base === start ? start : base;
  const commits = logRange(repo, compareStart, end, options.limit);
  const files = changedFiles(repo, compareStart, end);
  const areas = topLevelAreas(files);

  const lines = [
    "# Release Facts",
    "",
    `- Repository: \`${repo}\``,
    `- From: \`${startLabel}\``,
    `- To: \`${options.toRef}\` -> \`${end.slice(0, 12)}\``,
    `- Latest v* tag: \`${latest || "none"}\``,
    `- Merge base used: \`${compareStart.slice(0, 12)}\``,
    `- Commit count in range: \`${commits.length}\``,
    `- File count in range: \`${files.length}\``,
    "",
    "## Top-level areas",
    "",
  ];

  if (areas.length > 0) {
    for (const [area, count] of areas) {
      lines.push(`- \`${area}\`: ${count}`);
    }
  } else {
    lines.push("- none");
  }

  lines.push("", "## Commits", "");
  if (commits.length > 0) {
    for (const commit of commits) {
      lines.push(`- \`${commit.sha.slice(0, 12)}\` ${commit.subject}`);
    }
  } else {
    lines.push("- none");
  }

  lines.push("", "## Changed files", "");
  if (files.length > 0) {
    for (const filePath of files) {
      lines.push(`- \`${filePath}\``);
    }
  } else {
    lines.push("- none");
  }

  process.stdout.write(`${lines.join("\n")}\n`);
}

try {
  main();
} catch (error) {
  process.stderr.write(`error: ${error.message}\n`);
  process.exit(1);
}
