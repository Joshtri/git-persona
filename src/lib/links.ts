const WEBSITE_URL = "https://gitpersona.app";

export function changelogUrl(version: string): string {
  return `${WEBSITE_URL}/changelog#v${version.replace(/\./g, "-")}`;
}
