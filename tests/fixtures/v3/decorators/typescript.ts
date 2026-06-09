// TypeScript decorator fixtures
@Controller('/api')
class ApiController {
  @Get('/status')
  getStatus() {}

  @Post('/data', { validate: true })
  createData() {}
}

@Injectable()
class UserService {
  getUser(id: string) { return { id }; }
}

function log(target: any, key: string) {
  console.log(`${key} called`);
}

class Decorated {
  @log
  method() {}
}
