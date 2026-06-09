// TypeScript interface/type fixtures
interface User {
  id: string;
  email: string;
  name?: string;
  createdAt: Date;
}

type Post = {
  id: string;
  title: string;
  content: string;
  author: User;
  tags: string[];
};

interface Config {
  port: number;
  host: string;
  debug: boolean;
}
