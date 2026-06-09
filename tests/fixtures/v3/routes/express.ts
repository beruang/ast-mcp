// Express route fixtures
import express from 'express';
const app = express();
const router = express.Router();

app.get('/users', (req, res) => { res.json([]); });
app.post('/users', async (req, res) => { res.json({}); });
app.put('/users/:id', updateUser);
app.patch('/users/:id', updateUser);
app.delete('/users/:id', deleteUser);

router.get('/posts', listPosts);
router.post('/posts', createPost);

app.route('/items')
  .get(listItems)
  .post(createItem);

server.route({ method: 'GET', url: '/health', handler: healthCheck });
