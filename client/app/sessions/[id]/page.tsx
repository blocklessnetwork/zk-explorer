async function getSessionDetail(id: string): Promise<any> {
	const res = await fetch(`http://localhost:3005/api/proofs/${id}`)
	return await res.json()
}

export default async function SessionDetail({ params }: { params: { id: string } }) {
	const session = await getSessionDetail(params.id)
	return (
		<div className="h-full flex-1 flex-col md:flex">
			<div className="container flex flex-1 flex-col items-start space-y-2 py-4 md:h-16">
				{JSON.stringify(session)}
			</div>
		</div>
	)
}
