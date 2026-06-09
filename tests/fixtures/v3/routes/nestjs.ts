// NestJS controller route fixtures
@Controller('/users')
class UserController {
  @Get()
  listUsers() {}

  @Get('/:id')
  getUser() {}

  @Post()
  createUser() {}

  @Put('/:id')
  updateUser() {}

  @Delete('/:id')
  deleteUser() {}
}
