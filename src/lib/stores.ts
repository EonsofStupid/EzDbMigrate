import { writable } from 'svelte/store';

export type MigrationStatus = 'idle' | 'scanning' | 'migrating' | 'done' | 'error';

export interface MigrationState {
    status: MigrationStatus;
    progress: number;
    message: string;
    sourceUrl: string;
    destUrl: string;
}

export const migrationState = writable<MigrationState>({
    status: 'idle',
    progress: 0,
    message: 'Ready to migrate',
    sourceUrl: '',
    destUrl: '',
});

export function setMigrationStatus(status: MigrationStatus, message: string = '') {
    migrationState.update(s => ({ ...s, status, message }));
}

export function updateProgress(progress: number) {
    migrationState.update(s => ({ ...s, progress }));
}
