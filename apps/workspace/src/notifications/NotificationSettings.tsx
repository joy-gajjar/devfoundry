import type { NotificationStatus } from '../api/types'

interface Props {
  status: NotificationStatus
  onSetup: () => void
  onRevoke: () => void
}

export function NotificationSettings({ status, onSetup, onRevoke }: Props) {
  return <section aria-label="Notification settings" className="card-list">
    <h3>Notifications</h3>
    <p>{status.revoked ? 'Notifications are revoked' : status.enabled ? (status.paired ? 'Notifications are paired' : 'Notifications are enabled') : 'Notifications are disabled'}</p>
    {!status.enabled && <button type="button" onClick={onSetup}>Set up notifications</button>}
    {(status.enabled || status.paired) && <button type="button" onClick={onRevoke}>Revoke notifications</button>}
  </section>
}
