// React hooks test fixtures
import { useState, useEffect, useMemo, useCallback, useRef, useReducer, useContext } from 'react';

function useCustomHook(initial: string) {
  const [value, setValue] = useState(initial);
  useEffect(() => { console.log(value); }, [value]);
  return { value, setValue };
}

const useAnotherHook = () => {
  const ref = useRef(null);
  const [count, setCount] = useState(0);
  return { ref, count, setCount };
};

export function useFeatureFlag(name: string) {
  const [enabled, setEnabled] = useState(false);
  useEffect(() => { checkFlag(name).then(setEnabled); }, [name]);
  return enabled;
}

function MyComponent() {
  const [name, setName] = useState('');
  const items = useMemo(() => [name], [name]);
  const handleClick = useCallback(() => { setName(''); }, []);
  const flag = useCustomHook('test');
  const feature = useFeatureFlag('new-ui');
  return { name, items, handleClick, flag, feature };
}
