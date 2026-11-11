import { NextResponse, type NextRequest } from "next/server";

const SESSION_COOKIE = "research_apply_session";

export function proxy(request: NextRequest) {
  if (isPublic(request.nextUrl.pathname)) {
    return NextResponse.next();
  }
  const session = request.cookies.get(SESSION_COOKIE)?.value.trim();
  if (!session) {
    return NextResponse.redirect(new URL("/login", request.url));
  }
  return NextResponse.next();
}

function isPublic(pathname: string) {
  return pathname === "/" || pathname === "/login" || pathname === "/403";
}

export const config = {
  matcher: ["/((?!api|_next/static|_next/image|favicon.ico|.*\\.(?:svg|png|jpg|jpeg|gif|webp|ico)$).*)"],
};
