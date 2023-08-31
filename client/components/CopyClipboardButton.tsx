'use client'

import { ClipboardIcon } from 'lucide-react'
import { Button } from './ui/button'
import { toast } from './ui/use-toast'

export default function CopyClipboardButton({ text }: { text: string }) {
	return (
		<>
			<Button
				variant="ghost"
				size="icon"
				className="px-2 h-8 w-8"
				onClick={() => {
					navigator.clipboard.writeText(text)

					toast({
						title: 'Copied to Clipboard!'
					})
				}}
			>
				<ClipboardIcon className="h-4 w-4" />
			</Button>
		</>
	)
}
