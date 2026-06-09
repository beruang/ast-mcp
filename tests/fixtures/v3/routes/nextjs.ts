// Next.js App Router route handlers
export async function GET(request: Request) {
  return Response.json({ users: [] });
}

export async function POST(request: Request) {
  const body = await request.json();
  return Response.json(body, { status: 201 });
}

export async function PUT(request: Request) {
  return Response.json({ updated: true });
}

export async function DELETE(request: Request) {
  return Response.json({ deleted: true });
}
