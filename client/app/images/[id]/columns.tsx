'use client'

import { shortenString } from '@/utils/strings'
import { ColumnDef } from '@tanstack/react-table'
import dayjs from 'dayjs'

// This type is used to define the shape of our data.
// You can use a Zod schema here if you want.
export type ProofRecord = {
	id: string
	session_id: string
	status: 'preparing' | 'in-progress' | 'completed' | 'failed'

	receipt_cid: string
	created_at: string
	completed_at: string
}

export const columns: ColumnDef<ProofRecord>[] = [
	{
		accessorKey: 'session_id',
		header: 'ID'
	},
	{
		accessorKey: 'status',
		header: 'Status'
	},
	{
		accessorKey: 'created_at',
		header: 'Start',
		accessorFn: (v) => dayjs(v.created_at).format('MMM D, YYYY h:mm A')
	},
	{
		header: 'Duration',
		accessorFn: (v) =>
			v.completed_at ? `${dayjs(v.completed_at).diff(dayjs(v.created_at), 'second')}s` : 'N/A'
	},
	{
		accessorKey: 'receipt_cid',
		header: 'Proof Receipt',
		accessorFn: (v) => (v.receipt_cid ? shortenString(v.receipt_cid) : 'N/A')
	}
]
