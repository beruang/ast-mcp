// React component fixtures
import React from 'react';

export function UserCard(props: UserCardProps) {
  const [name, setName] = useState('');
  useEffect(() => { loadUser(); }, []);
  return <div className="user-card">{name}</div>;
}

const UserList = ({ users }: Props) => {
  const [filter, setFilter] = useState('');
  const filtered = useMemo(() => users.filter(u => u.name.includes(filter)), [users, filter]);
  return <div>{filtered.map(u => <UserCard key={u.id} user={u} />)}</div>;
};

export default function UserPage() {
  return <UserList users={[]} />;
}

const UserForm = memo(function UserForm(props: FormProps) {
  return <form><input /></form>;
});

const UserAvatar = forwardRef<HTMLDivElement, AvatarProps>((props, ref) => (
  <div ref={ref}><img src={props.src} /></div>
));

class UserWidget extends React.Component<WidgetProps> {
  render() { return <div>{this.props.title}</div>; }
}
