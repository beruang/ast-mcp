// TypeScript dependency edge fixtures
import { useState } from 'react';
import type { User } from './types';
import { getUser, type GetUserResult } from '../services/user';
import * as utils from '@/lib/utils';
import './styles.css';
import React from 'react';

export { UserCard } from './components';
export type { UserProps } from './types';
export { getUser as default } from './services';

const path = require('path');
const data = require('./data.json');
